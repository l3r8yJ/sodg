// Copyright (c) 2022-2023 Yegor Bugayenko
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included
// in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NON-INFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use crate::DeadRelay;
use crate::Sodg;
use anyhow::Result;
use log::trace;
use std::collections::{HashMap, HashSet};

impl Sodg {
    /// Take a slice of the graph, keeping only the vertex specified
    /// by the locator and its kids, recursively found in the entire graph.
    pub fn slice(&self, loc: &str) -> Result<Sodg> {
        let g = self.slice_some(loc, |_v, _to, _a| true)?;
        trace!(
            "#slice: taken {} vertices out of {} by '{}' locator",
            self.vertices.len(),
            g.vertices.len(),
            loc
        );
        Ok(g)
    }

    /// Take a slice of the graph, keeping only the vertex specified
    /// by the locator and its kids, recursively found in the entire graph,
    /// but only if the provided predicate agrees with the selection of
    /// the kids.
    pub fn slice_some(&self, loc: &str, p: impl Fn(u32, u32, String) -> bool) -> Result<Sodg> {
        let mut todo = HashSet::new();
        let mut done = HashSet::new();
        todo.insert(self.find(0, loc, &DeadRelay::default())?);
        loop {
            if todo.is_empty() {
                break;
            }
            let before: Vec<u32> = todo.drain().collect();
            for v in before {
                done.insert(v);
                let vtx = self.vertices.get(&v).unwrap();
                for e in vtx.edges.iter() {
                    if done.contains(&e.to) {
                        continue;
                    }
                    if !p(v, e.to, e.a.clone()) {
                        continue;
                    }
                    done.insert(e.to);
                    todo.insert(e.to);
                }
            }
        }
        let mut new_vertices = HashMap::new();
        for (v, vtx) in self.vertices.iter().filter(|(v, _)| done.contains(v)) {
            new_vertices.insert(*v, vtx.clone());
        }
        let g = Sodg {
            vertices: new_vertices,
            next_v: self.next_v,
            alerts: self.alerts.clone(),
            alerts_active: self.alerts_active,
            #[cfg(feature = "sober")]
            finds: HashSet::new(),
        };
        trace!(
            "#slice_some: taken {} vertices out of {} by '{}' locator",
            self.vertices.len(),
            g.vertices.len(),
            loc
        );
        Ok(g)
    }
}

#[test]
fn makes_a_slice() -> Result<()> {
    let mut g = Sodg::empty();
    g.add(0)?;
    g.add(1)?;
    g.bind(0, 1, "foo")?;
    g.add(2)?;
    g.bind(0, 2, "bar")?;
    assert_eq!(1, g.slice("foo")?.vertices.len());
    assert_eq!(1, g.slice("bar")?.vertices.len());
    Ok(())
}

#[test]
fn makes_a_partial_slice() -> Result<()> {
    let mut g = Sodg::empty();
    g.add(0)?;
    g.add(1)?;
    g.bind(0, 1, "foo")?;
    g.add(2)?;
    g.bind(1, 2, "bar")?;
    let slice = g.slice_some("foo", |_v, _to, _a| false)?;
    assert_eq!(1, slice.vertices.len());
    Ok(())
}

#[test]
fn skips_some_vertices() -> Result<()> {
    let mut g = Sodg::empty();
    g.add(0)?;
    g.add(1)?;
    g.bind(0, 1, "foo")?;
    g.add(2)?;
    g.bind(0, 2, "+bar")?;
    let slice = g.slice_some("ν0", |_, _, a| !a.starts_with('+'))?;
    assert_eq!(2, slice.vertices.len());
    Ok(())
}
