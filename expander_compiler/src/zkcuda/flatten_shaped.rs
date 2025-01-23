use crate::field::FieldRaw;

pub trait FlattenShaped<T: Clone> {
    fn flatten_shaped(&self, to: &mut Vec<T>) -> Vec<usize>;
}

impl<T: FieldRaw> FlattenShaped<T> for T {
    fn flatten_shaped(&self, to: &mut Vec<T>) -> Vec<usize> {
        to.push(self.clone());
        vec![]
    }
}

impl<T: Clone, V> FlattenShaped<T> for Vec<V>
where
    V: FlattenShaped<T>,
{
    fn flatten_shaped(&self, to: &mut Vec<T>) -> Vec<usize> {
        if self.is_empty() {
            panic!("Empty vector is not allowed for flatten");
        }
        let sub_shape = self[0].flatten_shaped(to);
        for v in self.iter().skip(1) {
            let cur_shape = v.flatten_shaped(to);
            if cur_shape != sub_shape {
                panic!("Shape mismatch in flatten");
            }
        }
        let mut shape = sub_shape;
        shape.push(self.len());
        shape
    }
}

pub fn flatten_shaped<T: FieldRaw, V: FlattenShaped<T>>(v: &V) -> (Vec<T>, Vec<usize>) {
    let mut to = Vec::new();
    let shape = v.flatten_shaped(&mut to).into_iter().rev().collect();
    (to, shape)
}
