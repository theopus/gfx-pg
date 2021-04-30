
pub mod file_system {
    use std::{fs, io};
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::path::PathBuf;
    use crate::utils::variables::get_str_r;


    pub fn read_lines_r(p: &[&str]) -> impl Iterator<Item=Result<String, io::Error>> {
        let root_dir = get_str_r("RX_ROOT");
        let path = path_to(&root_dir, p);
        read_lines(&path)
    }

    pub fn path_from_root(p: &[&str]) -> PathBuf{
        let root_dir = get_str_r("RX_ROOT");
        path_to(&root_dir, p)
    }

    pub fn read_file(p: &[&str]) -> Vec<u8> {
        fs::read(path_from_root(p)).unwrap()
    }

    pub fn path_to(root: &str, p: &[&str]) -> PathBuf {
        println!("{:?} - {:?}", root, p);
        let mut path = fs::canonicalize(root).unwrap();
        for i in p {
            path.push(i);
        }
        path
    }

    pub fn read_lines(path: &PathBuf) -> impl Iterator<Item=Result<String, io::Error>> {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        reader.lines()
    }
}
pub mod variables {
    use std::env;

    #[allow(unused_imports)]
    use log::{debug, error, info, trace, warn};

    #[allow(dead_code)]
    pub fn get_str(key: &str) -> Option<String> {
        _get_str(key, false)
    }

    pub fn get_str_r(key: &str) -> String {
        _get_str(key, true).unwrap()
    }

    fn _get_str(key: &str, required: bool) -> Option<String> {
        let result = env::var(key);
        return match result {
            Err(e) => {
                warn!("Key={}, err={}", key, e);
                if required {
                    panic!("Variable {} is required.", key);
                }
                None
            }
            Ok(v) => Some(v),
        };
    }

}
