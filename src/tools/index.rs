use crate::{
    errors::WGAError,
    parser::{common::Strand, maf::MAFReader},
};
use anyhow::anyhow;
use itertools::enumerate;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs::File,
    io::{Seek, Write},
};

pub fn build_index(
    mafreader: &mut MAFReader<File>,
    idx_wtr: Box<dyn Write>,
) -> Result<(), WGAError> {
    // init a MAfIndex2 struct
    let mut idx: MafIndex = HashMap::new();

    loop {
        let offset = mafreader.inner.stream_position()?;
        let record = mafreader.records().next();
        let record = match record {
            Some(r) => r?,
            None => break,
        };

        let mut name_vec = Vec::new();
        for (ord, sline) in enumerate(record.slines) {
            let name = sline.name;
            if !name_vec.contains(&name) {
                name_vec.push(name.clone());
            } else {
                return Err(WGAError::DuplicateName(name));
            }
            let start = sline.start;
            let end = sline.start + sline.align_size;
            let size = sline.size;
            let strand = sline.strand;

            // idx.entry(name.clone()).or_insert(MafIndexItem {
            //     ivls: Vec::new(),
            //     size,
            //     ord,
            // });

            if !idx.contains_key(&name) {
                idx.insert(
                    name.clone(),
                    MafIndexItem {
                        ivls: Vec::new(),
                        size,
                        ord,
                    },
                );
            } else {
                // compare ord if same
                if idx
                    .get(&name)
                    .ok_or(WGAError::Other(anyhow!("not excepted")))?
                    .ord
                    != ord
                {
                    return Err(WGAError::Other(anyhow!(
                        "There is a different order between Records!"
                    )));
                }
            }

            idx.get_mut(&name)
                .ok_or(WGAError::Other(anyhow!("not excepted")))?
                .ivls
                .push(IvP {
                    start,
                    end,
                    strand,
                    offset,
                });
        }
    }
    // write index to file if not empty
    if !idx.is_empty() {
        serde_json::to_writer(idx_wtr, &idx)?
    } else {
        return Err(WGAError::EmptyRecord);
    }
    Ok(())
}

pub type MafIndex = HashMap<String, MafIndexItem>;

#[derive(Debug, Serialize, Deserialize)]
pub struct MafIndexItem {
    pub ivls: Vec<IvP>,
    pub size: u64,
    pub ord: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IvP {
    pub start: u64,
    pub end: u64,
    pub strand: Strand,
    pub offset: u64,
}
