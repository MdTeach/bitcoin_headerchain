pub struct RecentBlockTimeStamp {
    data: Vec<u32>,
}

impl RecentBlockTimeStamp {
    pub fn new(data: &[u32]) -> Self {
        Self {
            data: data.to_vec(),
        }
    }

    pub fn output(&self) -> [u32; 11] {
        self.data.clone().try_into().expect("Incorrect length")
    }

    pub fn get_median_time(&self) -> u32 {
        let mut permute_data = self.data.clone();
        permute_data.sort();

        let nth_term = (permute_data.len() + 1) / 2;
        permute_data[nth_term]
    }

    pub fn insert_timestamp(&mut self, new_timestamp: u32) {
        self.data.pop();
        self.data.insert(0, new_timestamp);
    }
}

impl From<[u32; 11]> for RecentBlockTimeStamp {
    fn from(array: [u32; 11]) -> Self {
        RecentBlockTimeStamp {
            data: Vec::from(array),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_timestamp() {
        let mut rbt = RecentBlockTimeStamp::from([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
        rbt.insert_timestamp(12);
        assert_eq!(rbt.output(), [12, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn test_get_median_time() {
        let rbt = RecentBlockTimeStamp::from([50, 40, 30, 20, 10, 5, 15, 25, 35, 45, 55]);
        assert_eq!(rbt.get_median_time(), 35);
    }

    #[test]
    fn test_output() {
        let rbt = RecentBlockTimeStamp::from([10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110]);
        assert_eq!(rbt.output(), [10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110]);
    }

    #[test]
    fn test_initialization() {
        let data = [5; 11];
        let rbt = RecentBlockTimeStamp::new(&data);
        assert_eq!(rbt.output(), data);
    }
}
