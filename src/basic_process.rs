use crate::file::File;

pub fn remove_dupl(f: &mut File) {
    let mut i = 0;
    while i < f.content.len() {
        let mut j = i + 1;
        while j < f.content.len() {
            if f.get(i).content == f.get(j).content {
                f.remove(j);
            } else {
                j += 1;
            }
        }
        i += 1;
    }
}
