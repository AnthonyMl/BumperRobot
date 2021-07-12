#![allow(clippy::many_single_char_names)]
#![allow(clippy::nonstandard_macro_braces)]


use std::time::Instant;

use nalgebra::{DMatrix, DVector, dmatrix, dvector};


// TODO:
// - noise matrices R, Q need to take dt into account as well if varies a lot
// - initialize x and P with values based on data
// - try
//      - fixed lag smoothing
//      - fading memory
//      - adaptively increase process noise when bouncing off a wall
//      - adaptively increase process noise when the residual is large
//      - adaptively increase measurement noise when we get visual overlap
//      - multiple model variant


// uses observed 2d position and hidden 2d velocity
pub struct KalmanFilter {
    // state mean in [x, x', y, y'] format
    x: DVector<f32>,

    // system covariance
    p: DMatrix<f32>,

    // process noise
    q: DMatrix<f32>,

    // measurement function (extracts (and possibly converts) observed values)
    h: DMatrix<f32>,

    // measurement noise (covariance matrix)
    r: DMatrix<f32>,

    t_last: Instant,
}

impl KalmanFilter {
    pub fn default() -> KalmanFilter {
        let x = dvector![0.0, 0.0, 0.0, 0.0];

        let p = {
            // position variance (assumed equal in x and y)
            const PV: f32 = 500.0;
            // velocity variance (assumed equal in x and y)
            const VV: f32 = 500.0;

            dmatrix![
                 PV, 0.0, 0.0, 0.0;
                0.0,  VV, 0.0, 0.0;
                0.0, 0.0,  PV, 0.0;
                0.0, 0.0, 0.0,  VV]
        };

        let q = {
            // higher => trust data more, prediction less
            const Q: f32 = 6.0;
            dmatrix![
                0.0,   Q, 0.0, 0.0;
                  Q,   Q, 0.0, 0.0;
                0.0, 0.0, 0.0,   Q;
                0.0, 0.0,   Q,   Q]
        };

        let h = dmatrix![
            1.0, 0.0, 0.0, 0.0;
            0.0, 0.0, 1.0, 0.0];

        let r = {
            // higher => trust prediction more, data less
            const R: f32 = 14.0;
            dmatrix![
                R, 0.0;
                0.0, R]
        };

        let t_last = Instant::now();

        KalmanFilter { x, p, q, h, r, t_last }
    }

    // x = F * x + B * u
    // P = F * P * F^T + Q
    pub fn predict(&mut self) {
        let t = Instant::now();
        let dt = t.duration_since(self.t_last).as_secs_f32();
        self.t_last = t;

        // transition function
        let f = dmatrix![
            1.0,  dt, 0.0, 0.0;
            0.0, 1.0, 0.0, 0.0;
            0.0, 0.0, 1.0,  dt;
            0.0, 0.0, 0.0, 1.0];

        self.x = &f * &self.x;
        self.p = &f * &self.p * f.transpose() + &self.q;
    }

    // z measurement
    //----------------------
    // y = z - H*x
    // K = P * H^T * (H*P*H^T + R)^-1
    // x = x + K * y
    // P = (I - K*H) * P
    pub fn update(&mut self, z: &(f32, f32)) {
        let z = dvector![z.0, z.1];
        // residual
        let y = z - &self.h * &self.x;

        let pht = &self.p * self.h.transpose();
        let k = &pht * (&self.h * &pht + &self.r).try_inverse().unwrap();

        self.x = &self.x + &k * y;

        self.p = &self.p - &k * &self.h * &self.p;
    }

    pub fn position(&self) -> (f32, f32) {
        (self.x[0], self.x[2])
    }
}
