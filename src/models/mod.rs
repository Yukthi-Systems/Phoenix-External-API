use crate::models::errors::AppError;
use tokio_postgres::row::Row;

pub mod departments;
pub mod initial;
pub mod api_key;
pub mod domain;
pub mod errors;
pub mod orgs;


// Generic type for the list endpoint responses
#[derive(serde::Serialize)]
pub struct ListResponse<T> {
    pub items: Vec<T>,
    pub total: usize,
    pub current_count: usize,
    pub current_page: usize,
    pub total_pages: usize,
}


// Generic type for Query parameters
#[derive(serde::Deserialize)]
pub struct QueryParams {
    pub limit: i64,
    pub offset: i64,
}


impl<T> ListResponse<T> {
    pub fn from_rows<F>(rows: Vec<Row>, limit: i64, offset: i64, mapper: F) -> Self
    where
        F: Fn(Row) -> T,
    {
        let total = rows
            .first()
            .map(|row| row.get::<_, i64>("total_count") as usize)
            .unwrap_or(0);

        let items: Vec<T> = rows.into_iter().map(mapper).collect();
        let current_count = items.len();

        let limit = limit.max(1) as usize;
        let offset = offset.max(0) as usize;

        let current_page = (offset / limit) + 1;
        let total_pages = total.div_ceil(limit);

        Self {
            items,
            total,
            current_count,
            current_page,
            total_pages,
        }
    }
}


impl QueryParams {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.limit <= 0 {
            return Err(AppError::BadRequest("Limit must be greater than 0".into()));
        }

        if self.offset < 0 {
            return Err(AppError::BadRequest("Offset cannot be negative".into()));
        }

        if self.limit > 100 {
            return Err(AppError::BadRequest("Limit cannot be greater than 100".into()));
        }

        Ok(())
    }
}
