
// #[derive(Debug, Clone, Copy)]
// pub struct Ident {
//     id: usize,
//     kind: usize,
// }
//
// // this isn't really a map... but also keeps track of "side data"
// #[derive(Debug, Clone, Copy)]
// pub struct Idents {
//     pub latest: Ident,
//     pub kinds: [Option<usize>; NAMED_KINDS],
// }
//
// impl Idents {
//     pub fn new(latest: Ident) -> Self {
//         let mut s = Self {
//             latest,
//             kinds: [None; NAMED_KINDS],
//         };
//         s.insert(latest);
//         s
//     }
//     pub fn insert(&mut self, i: Ident) {
//         self.latest = i;
//         self.kinds[i.kind] = Some(i.id);
//     }
//     pub fn get(&self) -> Option<ID<K>> {
//         self.kinds[K::INDEX].map(|i| i.into())
//     }
// }
