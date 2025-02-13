use chrono::{Datelike, NaiveDate};

pub fn get_last_month(date: NaiveDate) -> NaiveDate {
    let (year, month) = if date.month() == 1 {
        (date.year() - 1, 12)
    } else {
        (date.year(), date.month() - 1)
    };

    // 获取当前月的第一天
    NaiveDate::from_ymd_opt(year, month, 1).unwrap()
}

#[cfg(test)]
mod test {
    use chrono::NaiveDate;

    use crate::date::get_last_month;

    #[test]
    pub fn test_get_last_month() {
        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        let test1 = get_last_month(date);
        assert_eq!(test1, NaiveDate::from_ymd_opt(2024, 12, 1).unwrap());
        let date2 = NaiveDate::from_ymd_opt(2024, 10, 1).unwrap();
        let test2 = get_last_month(date2);
        assert_eq!(test2, NaiveDate::from_ymd_opt(2024, 9, 1).unwrap());
    }
}
