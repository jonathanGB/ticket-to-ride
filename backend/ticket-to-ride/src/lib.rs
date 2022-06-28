pub mod city;
pub mod game_phase;
mod game_state;
pub mod map;
pub mod train_color;

#[macro_use]
extern crate smallvec;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
