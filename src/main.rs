extern crate rand;

use std::fmt::Debug;
use std::time;
use std::thread;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct Lifegame {
    size: [usize; 2],
    cells: Vec<Vec<bool>>,
}

impl Lifegame {
    fn new(s: [usize; 2], cs: Option<&[[usize; 2]]>) -> Self {
        let mut cells = Vec::new();
        for y in 0..s[0] {
            cells.push(Vec::new());
            for x in 0..s[1] {
                if let Some(cs) = cs {
                    if cs.iter().any(|c| y == c[0] && x == c[1]) {
                        cells[y].push(true);
                    } else {
                        cells[y].push(false);
                    }
                } else {
                    cells[y].push(false);
                }
            }
        }

        Lifegame {
            size: s,
            cells: cells,
        }
    }

    fn random(&mut self) {
        for y in 0..self.size[0] {
            for x in 0..self.size[1] {
                if rand::random::<usize>() % 2 == 1 {
                    self.cells[y][x] = true;
                } else {
                    self.cells[y][x] = false;
                }
            }
        }

    }

    fn display(&self) {
        let mut s = String::new();
        for y in 0..self.size[0] {
            for x in 0..self.size[1] {
                s.push(if self.cells[y][x] { 'O' } else { ' ' });
            }
            s += "\n"
        }
        print!("{}", s);
    }

    fn next(&mut self) {
        let prev = self.cells.clone();
        for y in 0..self.size[0] as i32 {
            for x in 0..self.size[1] as i32 {
                let mut a = 0;
                for ay in -1..2 {
                    for ax in -1..2 {
                        if ay != 0 || ax != 0 {
                            let yy = y + ay;
                            let xx = x + ax;
                            if yy >= 0 && yy < self.size[0] as i32 && xx >= 0 &&
                               xx < self.size[1] as i32 {
                                if prev[yy as usize][xx as usize] {
                                    a += 1;
                                }
                            }
                        }
                    }
                }

                let y = y as usize;
                let x = x as usize;
                let c = prev[y][x];

                if (c && a == 2) || (c && a == 3) {
                    //self.cells[y][x] = true
                } else if !c && a == 3 {
                    self.cells[y][x] = true
                } else if c {
                    self.cells[y][x] = false
                };
            }
        }
    }

    fn next_parallel_channel(&mut self) {
        let s = Arc::new(self.clone());
        let (tx, rx) = mpsc::channel();

        for y in 0..self.size[0] {
            for x in 0..self.size[1] {
                let tx = tx.clone();
                let s = s.clone();
                thread::spawn(move || {
                    let mut a = 0;
                    for ay in -1..2 as i32 {
                        for ax in -1..2 as i32 {
                            if ay != 0 || ax != 0 {
                                let yy = y as i32 + ay;
                                let xx = x as i32 + ax;
                                if yy >= 0 && yy < s.size[0] as i32 && xx >= 0 &&
                                   xx < s.size[1] as i32 {
                                    if s.cells[yy as usize][xx as usize] {
                                        a += 1;
                                    }
                                }
                            }
                        }
                    }

                    let c = s.cells[y][x];

                    if (c && a == 2) || (c && a == 3) {
                        tx.send(([y, x], true)).unwrap();
                    } else if !c && a == 3 {
                        tx.send(([y, x], true)).unwrap();
                    } else {
                        tx.send(([y, x], false)).unwrap();
                    }
                });
            }
        }

        for y in 0..self.size[0] {
            for x in 0..self.size[1] {
                match rx.recv() {
                    Err(_) => return,
                    Ok((i, c)) => self.cells[i[0]][i[1]] = c,
                }
            }
        }
    }

    fn next_parallel_mutex(&mut self) {
        let prev = Arc::new(self.clone());
        let next = Arc::new(Mutex::new(self.clone()));
        let mut hs = Vec::new();

        for y in 0..self.size[0] {
            for x in 0..self.size[1] {
                let prev = prev.clone();
                let next = next.clone();
                let h = thread::spawn(move || {
                    let mut a = 0;
                    for ay in -1..2 as i32 {
                        for ax in -1..2 as i32 {
                            if ay != 0 || ax != 0 {
                                let yy = y as i32 + ay;
                                let xx = x as i32 + ax;
                                if yy >= 0 && yy < prev.size[0] as i32 && xx >= 0 &&
                                   xx < prev.size[1] as i32 {
                                    if prev.cells[yy as usize][xx as usize] {
                                        a += 1;
                                    }
                                }
                            }
                        }
                    }

                    let c = prev.cells[y][x];
                    let mut next = next.lock().unwrap();

                    if (c && a == 2) || (c && a == 3) {
                        //next.cells[y][x] = true;
                    } else if !c && a == 3 {
                        next.cells[y][x] = true;
                    } else if c {
                        next.cells[y][x] = false;
                    }
                });
                hs.push(h);
            }
        }

        for h in hs.into_iter() {
            h.join().unwrap();
        }

        self.cells = next.lock().unwrap().clone().cells;
    }

    fn eq(&self, cs: &Vec<Vec<bool>>) -> bool {
        for y in 0..self.size[0] {
            for x in 0..self.size[1] {
                if self.cells[y][x] != cs[y][x] {
                    return false;
                }
            }
        }

        true
    }

    fn subsequences<T: Clone>(xs: &Vec<T>) -> Vec<Vec<T>> {
        let mut result: Vec<Vec<T>> = vec![];
        for x in xs.iter() {
            let mut acc: Vec<Vec<T>> = vec![];
            for r in result.into_iter() {
                let mut rr = r.clone();
                rr.push(x.clone());
                acc.push(rr);
                acc.push(r);
            }
            acc.push(vec![x.clone()]);
            result = acc;
        }
        result
    }

    fn score(&mut self) -> usize {
        let mut n = 0;
        let mut cs = Vec::new();
        for _ in 0..10 {
            cs.push(self.cells.clone());
        }

        loop {
            n += 1;
            cs.remove(0);
            cs.push(self.cells.clone());
            self.next();
            for c in cs.iter() {
                if self.eq(&c) {
                    return n;
                }
            }
        }
    }

    fn run(&mut self, from: [usize; 2], to: [usize; 2]) -> usize {
        let mut ls = Vec::new();
        for y in from[0]..to[0] {
            for x in from[1]..to[1] {
                ls.push([y, x]);
            }
        }
        let ss = Self::subsequences(&ls);

        let mut best_score = 0;
        let mut best_cells = self.cells.clone();

        for ps in ss.into_iter() {
            for y in 0..self.size[0] {
                for x in 0..self.size[1] {
                    self.cells[y][x] = false;
                }
            }

            for p in ps.into_iter() {
                self.cells[p[0]][p[1]] = true;
            }

            let cells = self.cells.clone();
            let score = self.score();
            if best_score <= score {
                best_score = score;
                best_cells = cells;
            }

        }

        self.cells = best_cells;
        best_score
    }
}

fn main() {
    let mut n = 0;
    let mut lg = Lifegame::new([100, 100], None);
    lg.run([50, 50], [53, 53]);
    loop {
        n += 1;
        print!("\x1B[1;0H\x1B[0J{}", n);
        lg.display();
        lg.next();
        thread::sleep(time::Duration::from_millis(100));
    }
}
