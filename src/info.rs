//! Info attribute


/// Trait providing access to attached info
///
/// FIRRTL defines an optional info attribute for some entities. The info will
/// usually hold information about where (in a generator's source) the entity
/// was generated.
pub trait WithInfo {
    /// Retrieve info attribute
    ///
    /// If no info is attached to the entity, this function will return `None`
    fn info(&self) -> Option<&str>;

    /// Set the info attribute
    fn set_info(&mut self, info: Option<String>);

    /// Set the info attribute
    fn with_info(mut self, info: Option<String>) -> Self
    where Self: Sized
    {
        self.set_info(info);
        self
    }

    /// Clear the attached info
    fn clear_info(&mut self) {
        self.set_info(None)
    }
}

