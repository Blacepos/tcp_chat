// An experiment with function traits. This, and the functions in "helpers.rs" only serve the
// impractical role of saving a few lines in `main()`

pub type Validator<T> = fn (&T) -> bool;

/// Functions which can be repeated until they return valid data
pub trait UntilValid<T> {
    fn until_valid(&self, validator: Validator<T>) -> T;
}

impl<T, A> UntilValid<T> for A
where A: Fn() -> T
{
    fn until_valid(&self, validator: Validator<T>) -> T {
        loop {
            let x = self.call(());
    
            if validator(&x) {
                return x;
            }
        }
    }
}


/// Functions which can be repeated until they return data in the form of an Ok result
pub trait UntilResult<T> {
    fn until_ok(&self) -> T;
}

impl<T, A, E> UntilResult<T> for A
where A: Fn() -> Result<T, E>
{
    fn until_ok(&self) -> T {
        loop {
            let x = self.call(());

            if let Ok(val) = x {
                return val;
            }
        }
    }
}
