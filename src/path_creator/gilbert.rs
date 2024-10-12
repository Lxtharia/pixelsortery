// This algorithm is adapted from a Python implementation by [Author's Name].
// BSD 2-Clause License
// 
// Copyright (c) 2018, Jakub Červený
// All rights reserved.
// 
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are met:
// 
// * Redistributions of source code must retain the above copyright notice, this
//   list of conditions and the following disclaimer.
// 
// * Redistributions in binary form must reproduce the above copyright notice,
//   this list of conditions and the following disclaimer in the documentation
//   and/or other materials provided with the distribution.
// 
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
// DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
// FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
// DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
// SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
// CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
// OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

pub(crate) fn path_hilbert(width: u64, height: u64) -> Vec<Vec<u64>> {
    // Most of this code here is copied/translated from here: https://github.com/jakubcerveny/gilbert/blob/master/gilbert2d.py
    // Which i translated to C at some point and then translated that C code to rust

    fn sgn(x: i64) -> i64 {
        if x < 0 {
            -1
        } else if x > 0 {
            1
        } else {
            0
        }
    };

    /// To bring a bit of glitchiness into it, switch ay and bx as parameters (or any other and try experimenting!)
    fn generate2dhilbert(
        coords: &mut Vec<(u64, u64)>,
        mut x: i64,
        mut y: i64,
        ax: i64,
        ay: i64,
        bx: i64,
        by: i64,
    ) {
        // width and height
        let w = (ax + ay).abs();
        let h = (bx + by).abs();

        let dax = sgn(ax);
        let day = sgn(ay);
        let dbx = sgn(bx);
        let dby = sgn(by);


        if (h == 1) {
            // trivial row fill
            for i in 0..w {
                coords.push((x as u64, y as u64));
                x += dax;
                y += day;
            }
            return;
        }

        if (w == 1) {
            // trivial column fill
            for i in 0..h {
                coords.push((x as u64, y as u64));
                x += dbx;
                y += dby;
            }
            return;
        }

        let mut ax2 = ax / 2;
        let mut ay2 = ay / 2;
        let mut bx2 = bx / 2;
        let mut by2 = by / 2;

        let w2 = (ax2 + ay2).abs();
        let h2 = (bx2 + by2).abs();

        if (2 * w > 3 * h) {
            if (w2 % 2 != 0 && w > 2) {
                // prefer even steps
                ax2 += dax;
                ay2 += day;
            }

            // long case: split in two parts only
            generate2dhilbert(coords, x, y, ax2, ay2, bx, by);
            generate2dhilbert(coords, x + ax2, y + ay2, ax - ax2, ay - ay2, bx, by);
        } else {
            if (h2 % 2 != 0 && h > 2) {
                // prefer even steps
                bx2 += dbx;
                by2 += dby;
            }

            // standard case: one step up, one long horizontal, one step down
            generate2dhilbert(coords, x, y, bx2, by2, ax2, ay2);
            generate2dhilbert(coords, x + bx2, y + by2, ax, ay, bx - bx2, by - by2);
            generate2dhilbert(
                coords,
                x + (ax - dax) + (bx2 - dbx),
                y + (ay - day) + (by2 - dby),
                -bx2,
                -by2,
                -(ax - ax2),
                -(ay - ay2),
            );
        }
    };

    let mut path = Vec::new();
    if (width >= height) {
        generate2dhilbert(&mut path, 0, 0, width as i64, 0, 0, height as i64)
    } else {
        generate2dhilbert(&mut path, 0, 0, 0, height as i64, width as i64, 0)
    };

    vec![path.into_iter().map(|(x,y)| y*width+x).collect()]
}

