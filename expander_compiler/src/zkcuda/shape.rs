pub type Shape = Vec<usize>;
pub type Axes = Vec<usize>;

#[derive(Debug, Clone)]
pub struct ShapeHistory {
    vec_len: usize,
    entries: Vec<Entry>,
}

#[derive(Debug, Clone)]
struct Entry {
    shape: Shape,
    axes: Option<Axes>,
}

impl Entry {
    fn minimize(&self, keep_first_dim: bool) -> Self {
        let axes = match &self.axes {
            None => {
                let new_shape = if !keep_first_dim || self.shape.len() <= 1 {
                    vec![shape_vec_len(&self.shape)]
                } else {
                    vec![self.shape[0], shape_vec_len(&self.shape[1..])]
                };
                return Entry {
                    shape: new_shape,
                    axes: None,
                };
            }
            Some(axes) => axes,
        };
        let mut merge_shape = vec![false; self.shape.len()];
        for (i, (&a, &b)) in axes.iter().zip(axes.iter().skip(1)).enumerate() {
            if a + 1 == b && (!keep_first_dim || i > 0) {
                merge_shape[b] = true;
            }
        }
        let mut new_shape_id = vec![0; self.shape.len()];
        let mut new_shape = Vec::with_capacity(self.shape.len());
        for (i, &s) in self.shape.iter().enumerate() {
            if merge_shape[i] {
                new_shape_id[i] = new_shape.len();
                new_shape.push(s);
            } else {
                *new_shape.last_mut().unwrap() *= s;
            }
        }
        let mut new_axes = Vec::with_capacity(axes.len());
        for &a in axes {
            if !merge_shape[a] {
                new_axes.push(new_shape_id[a]);
            }
        }
        Entry {
            shape: new_shape,
            axes: Some(new_axes),
        }
    }
}

pub trait Reshape {
    fn reshape(&self, new_shape: &[usize]) -> Self;
}

pub trait Transpose {
    fn transpose(&self, axes: &[usize]) -> Self;
}

pub fn shape_vec_len(shape: &[usize]) -> usize {
    shape.iter().product()
}

impl ShapeHistory {
    pub fn new(initial_shape: Shape) -> Self {
        Self {
            vec_len: shape_vec_len(&initial_shape),
            entries: vec![Entry {
                shape: initial_shape,
                axes: None,
            }],
        }
    }

    // Suppose we need to ensure that first dimension of current shape can be split
    // This function returns a list of dimension lengths where the initial vector must be split
    pub fn get_initial_split_list(&self) -> Vec<usize> {
        panic!("TODO")
    }
}

impl Reshape for ShapeHistory {
    fn reshape(&self, new_shape: &[usize]) -> Self {
        if shape_vec_len(new_shape) != self.vec_len {
            panic!("Reshape to a shape with different vector length is not allowed, expected {}, got {}", self.vec_len, shape_vec_len(new_shape));
        }
        let mut res = self.clone();
        if res.entries.last().unwrap().axes.is_none() {
            res.entries.last_mut().unwrap().shape = new_shape.to_vec();
        } else {
            res.entries.push(Entry {
                shape: new_shape.to_vec(),
                axes: None,
            });
        }
        res
    }
}

impl Transpose for ShapeHistory {
    fn transpose(&self, axes: &[usize]) -> Self {
        if axes.len() != self.entries.last().unwrap().shape.len() {
            panic!(
                "Transpose axes length must match the shape length, expected {}, got {}",
                self.entries.last().unwrap().shape.len(),
                axes.len()
            );
        }
        let mut res = self.clone();
        match &res.entries.last().unwrap().axes {
            None => {
                res.entries.last_mut().unwrap().axes = Some(axes.to_vec());
            }
            Some(cur_axes) => {
                let new_axes: Vec<usize> = axes.iter().map(|&a| cur_axes[a]).collect();
                res.entries.last_mut().unwrap().axes = Some(new_axes);
            }
        }
        res
    }
}
