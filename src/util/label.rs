// this is not even remotely worth it but technically it doesn't use the heap I think xdddddddddd

use std::marker::PhantomData;
pub trait Labeler<S> = Fn(&mut std::fmt::Formatter<'_>, &S) -> std::fmt::Result;

pub trait Labelable<S> {
    fn labeled<L: Labeler<S>>(&self, l: L) -> Labeled<Self, L, S>
    where
        Self: Sized;
}

pub struct Labeled<'a, T, L: Labeler<S>, S> {
    data: &'a T,
    labeler: L,
    pd: PhantomData<S>,
}

pub trait LabeledFmt<S> {
    fn fmt_label(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        label: &dyn Labeler<S>,
    ) -> std::fmt::Result;
}

impl<T: LabeledFmt<S>, S> Labelable<S> for T {
    fn labeled<L: Labeler<S>>(&self, l: L) -> Labeled<Self, L, S> {
        Labeled {
            data: self,
            labeler: l,
            pd: PhantomData,
        }
    }
}

impl<T: LabeledFmt<S>, L: Labeler<S>, S> std::fmt::Debug for Labeled<'_, T, L, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.data.fmt_label(f, &self.labeler)
    }
}
