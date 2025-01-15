use chrono::{Datelike, NaiveDate};

pub fn get_last_month(date: NaiveDate) -> (i32, i32) {
    let new_month = if date.month() == 1 {
        12
    } else {
        date.month() as i32 - 1
    };
    let new_year = if new_month == 12 && date.month() == 1 {
        date.year() - 1
    } else {
        date.year()
    };
    (new_year, new_month)
}

#[cfg(test)]
mod test {
    use chrono::NaiveDate;

    use super::get_last_month;

    #[test]
    pub fn test() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let test1 = get_last_month(date);
        assert_eq!(test1, (2024, 12));
        let date2 = NaiveDate::from_ymd_opt(2024, 10, 1).unwrap();
        let test2 = get_last_month(date2);
        assert_eq!(test2, (2024, 9));
    }
}
