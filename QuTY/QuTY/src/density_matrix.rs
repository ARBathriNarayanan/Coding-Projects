#[allow(dead_code)]
pub struct DensityMatrixSimulator {
    pub dim: usize,
    pub matrix_re: Vec<Vec<f64>>,
    pub matrix_im: Vec<Vec<f64>>,
}

#[allow(dead_code)]
impl DensityMatrixSimulator {
    pub fn new(num_qubits: usize) -> Self {
        let dim = 1 << num_qubits;
        let mut matrix_re = vec![vec![0.0; dim]; dim];
        let matrix_im = vec![vec![0.0; dim]; dim];
        matrix_re[0][0] = 1.0;

        Self { dim, matrix_re, matrix_im }
    }

    pub fn trace(&self) -> f64 {
        let mut tr = 0.0;
        for i in 0..self.dim {
            tr += self.matrix_re[i][i];
        }
        tr
    }
}
