/// Conflict defines API to check if two vars conflicts with each other.
pub trait Conflict {
    fn conflict(&self, with: &Self) -> bool;
}
