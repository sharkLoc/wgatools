use crate::errors::ParseError;
use crate::parser::common::{FileFormat, Strand};
use itertools::Itertools;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader, Read};

/// Parser for MAF file format
pub struct MAFReader<R: io::Read> {
    inner: BufReader<R>,
    pub header: String,
}

impl<R> MAFReader<R>
where
    R: io::Read,
{
    /// Create a new PAF parser
    pub fn new(reader: R) -> Self {
        let mut buf_reader = BufReader::new(reader);
        let mut header = String::new();
        buf_reader.read_line(&mut header).unwrap();
        MAFReader {
            inner: buf_reader,
            header,
        }
    }

    /// Iterate over the records in the MAF file
    /// ```
    ///use wgalib::parser::maf::MAFReader;
    /// let mut reader = MAFReader::from_path("data/test.maf").unwrap();
    ///for record in reader.records() {
    ///    let record = record.unwrap();
    ///    println!("{:?}", record);
    ///}
    /// ```
    pub fn records(&mut self) -> MAFRecords<R> {
        MAFRecords {
            inner: self.inner.by_ref(),
        }
    }

    /// convert method
    pub fn convert(&mut self, _outputpath: &str, format: FileFormat) {
        match format {
            FileFormat::Chain => {}
            FileFormat::Blocks => {}
            _ => {}
        }
    }
}

impl MAFReader<File> {
    /// Create a new PAF parser from a file path
    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> io::Result<MAFReader<File>> {
        File::open(path).map(MAFReader::new)
    }
}

/// A MAF s-line refer to https://genome.ucsc.edu/FAQ/FAQformat.html#format5
// a score=111
// s ref    100 10 + 100000 ---AGC-CAT-CATT
// s contig 0   10 + 10     ---AGC-CAT-CATT
//
// a score=222
// s ref    100 12 + 100000 ---AGC-CAT-CATTTT
// s contig 0   12 + 12     ---AGC-CAT-CATTTT
#[derive(Debug, PartialEq)]
struct MAFSLine {
    mode: char,
    name: String,
    start: u64,
    align_size: u64,
    strand: Strand,
    size: u64,
    seq: String,
}

fn str2u64(s: &str) -> Result<u64, ParseError> {
    // TODO: move to common.rs module
    match s.parse::<u64>() {
        Ok(n) => Ok(n),
        Err(_) => Err(ParseError::new_parse_int_err(s)),
    }
}

fn parse_sline(line: String) -> Result<MAFSLine, ParseError> {
    let mut iter = line.split_whitespace();
    let mode = match iter.next() {
        Some(mode) => mode.chars().next().unwrap(), // TODO: error handling
        None => panic!("s-line mode is missing"),   // TODO: error handling
    };
    let name = match iter.next() {
        Some(name) => name.to_string(),
        None => panic!("s-line name is missing"), // TODO: error handling
    };
    let start = match iter.next() {
        Some(start) => str2u64(start)?,
        None => panic!("s-line start is missing"), // TODO: error handling
    };
    let align_size = match iter.next() {
        Some(align_size) => str2u64(align_size)?, // TODO: error handling
        None => panic!("s-line align_size is missing"), // TODO: error handling
    };
    let strand = match iter.next() {
        Some(strand) => Strand::from(strand), // TODO: error handling
        None => panic!("s-line strand is missing"), // TODO: error handling
    };
    let size = match iter.next() {
        Some(size) => str2u64(size)?,
        None => panic!("s-line size is missing"), // TODO: error handling
    };
    let seq = match iter.next() {
        Some(seq) => seq.to_string(),
        None => panic!("s-line seq is missing"), // TODO: error handling
    };
    if iter.next().is_some() {
        panic!("s-line has more than 8 fields")
    };
    Ok(MAFSLine {
        mode,
        name,
        start,
        align_size,
        strand,
        size,
        seq,
    })
}

fn sline_from_string(value: String) -> Result<MAFSLine, ParseError> {
    let s_line = parse_sline(value)?;
    Ok(s_line)
}

/// A MAF alignment record refer to https://genome.ucsc.edu/FAQ/FAQformat.html#format5
/// a pair of a-lines should be a align record
#[derive(Debug, PartialEq)]
pub struct MAFRecord {
    score: u64,
    slines: Vec<MAFSLine>,
}

/// A MAF record iterator
/// two s-lines should be a record
pub struct MAFRecords<'a, R: io::Read> {
    inner: &'a mut BufReader<R>,
}

impl Iterator for MAFRecords<'_, File> {
    type Item = Result<MAFRecord, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let score = 255;
        match self.inner.lines().next() {
            Some(Ok(line)) => {
                if !line.starts_with('s') {
                    self.next() // skip empty line
                } else {
                    // start read multi s-lines
                    let mut mafrecord = MAFRecord {
                        // init a maf-record
                        score,
                        slines: Vec::new(),
                    };
                    let sline = match sline_from_string(line) {
                        Ok(sline) => sline,
                        Err(e) => return Some(Err(e)),
                    };
                    mafrecord.slines.push(sline); // push first s-line
                                                  // start read next sequential s-lines
                    for line in self.inner.lines() {
                        match line {
                            Ok(line) => {
                                if line.starts_with('s') {
                                    let sline = match sline_from_string(line) {
                                        Ok(sline) => sline,
                                        Err(e) => return Some(Err(e)),
                                    };
                                    mafrecord.slines.push(sline);
                                } else {
                                    // if s-line is over, break
                                    break;
                                }
                            }
                            _ => {
                                // if line is empty, break
                                break;
                            }
                        }
                    }
                    Some(Ok(mafrecord))
                }
            }
            _ => None, // if line is empty, iterator over
        }
    }
}

fn cigar_cat(c1: &char, c2: &char) -> &'static str {
    if c1 == c2 {
        "M"
    } else if c1 == &'-' {
        "I"
    } else if c2 == &'-' {
        "D"
    } else {
        "M"
    }
}

impl MAFRecord {
    pub fn get_cigar(&self) -> String {
        let mut cigar = String::new();
        let seq1_iter = self.slines[0].seq.chars();
        let seq2_iter = self.slines[1].seq.chars();
        seq1_iter
            .zip(seq2_iter)
            .group_by(|(c1, c2)| cigar_cat(c1, c2))
            .into_iter()
            .for_each(|(k, g)| {
                let len = g.count();
                cigar.push_str(&len.to_string());
                cigar.push_str(k);
            });
        cigar
    }
}
