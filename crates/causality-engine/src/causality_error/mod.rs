// Create a local wrapper newtype for Box<dyn CausalityError>
pub struct BoxedCausalityError(pub Box<dyn CausalityError>);

// Now implement From<EngineError> for our local newtype
impl From<EngineError> for BoxedCausalityError {
    fn from(err: EngineError) -> Self {
        BoxedCausalityError(Box::new(err) as Box<dyn CausalityError>)
    }
}

// And implement From<Box<EngineError>> for our local newtype
impl From<Box<EngineError>> for BoxedCausalityError {
    fn from(err: Box<EngineError>) -> Self {
        BoxedCausalityError(Box::new(*err) as Box<dyn CausalityError>)
    }
}

// Deref implementation to make the newtype transparent
impl std::ops::Deref for BoxedCausalityError {
    type Target = Box<dyn CausalityError>;
    
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// DerefMut implementation to make the newtype transparent
impl std::ops::DerefMut for BoxedCausalityError {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

// Convert error trait - allow ? operator to work with Result<T, BoxedCausalityError>
pub trait ConvertError<T> {
    fn convert_error(self) -> Result<T, Box<dyn CausalityError>>;
}

impl<T, E: Into<BoxedCausalityError>> ConvertError<T> for std::result::Result<T, E> {
    fn convert_error(self) -> Result<T, Box<dyn CausalityError>> {
        self.map_err(|e| e.into().0)
    }
} 