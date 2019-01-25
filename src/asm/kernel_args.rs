#[derive(Default, Debug)]
pub struct KernelArg {
    pub name: String,
    pub size: u32,
    pub offset: u32
}

pub fn extract_kernel_args(section_note: &Vec<u8>) -> Vec<KernelArg> {
    let cl_note: Vec<u8> = section_note
        .iter()
        .skip_while(|&&c| c != '\n' as u8)
        .filter(|&&c| c != 0)
        .map(|c| c.to_owned()).collect();

    let metadata = String::from_utf8_lossy(cl_note.as_slice());
    let args_raw: Vec<Vec<String>> = metadata
        .lines()
        .skip_while(|l| !l.starts_with("    Args:")).skip(1)
        .take_while(|l| !l.starts_with("    CodeProps:"))
        .fold(Vec::new(), |mut args, l| {
            if l.starts_with("      - ") {
                args.push(vec![l[8..].replace(" ", "")])
            }
            else {
                args.last_mut().unwrap().push(l.replace(" ", ""))
            }
            args
        });

    let mut offset = 0;

    args_raw
        .into_iter()
        .map(|args| {
            let name = args.iter().find(|e| e.starts_with("Name")).map(|e| &e[5..])
                .or(args.iter().find(|e| e.starts_with("ValueKind")).map(|e| &e[10..]))
                .unwrap().to_string();
            let size = args.iter().find(|e| e.starts_with("Size")).unwrap()[5..]
                .parse::<u32>().unwrap();
            let alignment = args.iter().find(|e| e.starts_with("Align")).unwrap()[6..]
                .parse::<u32>().unwrap();
            offset += offset % alignment;
            offset += size;
            KernelArg { name, size, offset: offset - size }
        })
        .collect()
}

