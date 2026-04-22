// use sqlx_postgres::{PgColumn, PgRow, PgTypeInfo};
use sqlx_postgres::{ PgColumn, PgRow, PgTypeInfo};
use sqlx::{Column, Row as SqlxRow};


static TEXT_TYPE: &PgTypeInfo = &PgTypeInfo::with_name("Text");
static INT4_TYPE: &PgTypeInfo = &PgTypeInfo::with_name("Int4");
static NAME_TYPE: &PgTypeInfo = &PgTypeInfo::with_name("Name");

fn is_text(type_info: &PgTypeInfo) -> bool {
    type_info.type_eq(TEXT_TYPE)
}

fn is_int4(type_info: &PgTypeInfo) -> bool {
    type_info.type_eq(INT4_TYPE)
}

fn is_name(type_info: &PgTypeInfo) -> bool {
    type_info.type_eq(NAME_TYPE)
}

// converts a cell to a String depending on its type_info
fn stringify_type_info(type_info: &PgTypeInfo, col: &PgColumn, row: &PgRow) -> String {
            let mut out = String::new();

            // simply return if text or name
            if is_text(type_info) || is_name(type_info) {
                out = row.get(col.name());
                return out
            }
            
            // if int4, convert to string first
            if is_int4(type_info) {
                let data: i32 = row.get(col.name());
                let data_str_res = data.to_string();
                return data_str_res;
            } 

            out
}   


// converts a row to a vec of Strings depending on the type of corresponding columns
pub fn stringify(row: &PgRow) -> Vec<String> {

        let mut row_strs: Vec<String> = Vec::new();
        let cols = row.columns();
    
        for col in cols {

            let type_info = col.type_info();

            row_strs.push(stringify_type_info(type_info, col, row));
        }
        row_strs
}

