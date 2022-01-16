

How to read packets:
everything is written in little endien

first, some common things you'll see
 - Number Types:
  - `u8`: ugnsigned byte
  - `i8`: signed byte
  - `u16`: unsigned short (2 bytes)
  - `i16`: signed short (2 bytes)
  - `u32`: unsigned int (4 bytes)
  - `i32`: signed int (4 bytes)
  - `u64`: unsigned long (8 bytes)
  - `i64`: signed long (8 bytes)
  - `u128`: unsigned double long (16 bytes)
  - `i128`: signed double long (16 bytes)

 - Data Types
  - `String`: text
  - `bool`: boolean
  - `(T1, T2[,...])`: tuple. this can contain any number of values. ie `(u8, bool, String, String)`
  - `Vec<T>`: list/array of type T

 - Rust Enums
  - rust enums are unlike other language enums, they can contain variable data. this 


How types are written
 - any number type: type as LE bytes
 - `String` : [string length (u64)] [char1 (u8)] [char2 (u8)] [...]
 - `bool`   : [0 (false) or 1 (true) as u8]
 - `(v1,v2)`: [v1] [v2] [...]
 - `Vec<T>` : [list size (u64)] [data in index 0] [data in index 1] [...]


 How to read enums:
  - enum variant id (should be specified above the varient, and type should be specified above the enum declaration)
  - every field in order from top to bottom
  - ie. for the following:
  ```rs
  #[Packet(type=u16)]
  enum SomeEnum {
    #[Packet(id=10)]
    SomeVariant {value1:String, value2: u32}
  }
  let type_to_write = SomeEnum::SomeVariant {value1: String::from("this is some text"), value2: 3000}
  ```
  the data in `type_to_write` would be written like so:
  [packet_id] [value1] [value2]
  [0A,00] [11,00,00,00,00,00,00,00][74,68,69,73,20,69,73,20,73,6f,6d,65,20,74,65,78,74] [b8, 0b, 00, 00]