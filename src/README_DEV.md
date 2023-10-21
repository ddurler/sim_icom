# Sim_ICOM conversions

## TLVFrame

DataFrame -> String: format!(data_frame)
RawFrame -> DataFrame : DataFrame::try_from(&raw_frame)
DataFrame -> tag : data_frame.get_tag
DataFrame -> Vec<DataItem>: data_frame.get_data_items
Vec<u8> -> DataItem: DataItem::decode(&[u8])
Vec<u8> -> Vec<DataItem>: DataItem::decode_all
DataItem -> Vec<u>: data_item.encode
FrameState -> String: format!
RawFrame -> String: format!
Vec<u8> -> RawFrame: new, raw_frame.push, raw_frame.extend
DataItem +-> RawFrame: raw_frame.try_extend
RawFrame -> FrameState: raw_frame.get_state
RawFrame -> Vec<u8>: raw_frame.encode

## TData

TFormat, Vec<u8> -> TValue: be_format::decode
TValue -> Vec<u8>: be_format::encode
Vec<u8> -> String: vec_u8_to_string(&vec_u8)
String -> Vec<u8>: string_to_vec_u8(string)
TFormat -> String: format!(t_format)
u8 -> TFormat: TFormat::from(u8)
TFormat -> u8: u8::from(t_format)
TValue -> String: format!(t_value)
TValue -> TFormat: TFormat::from(&t_value)
<type> -> TFormat: <type>::from(t_format)
TValue(type) -> TValue(autre_type): t_format.to_t_format_<autre_type>
TValue -> Vec<u8> -> t_value.to_vec_u8
Vec<u8> -> TValue: via DataItem::decode | via be_data::decode

## Database

IdTag -> String: format!(id_tag)
Database -> String: format!(database)
String -> Database: Database::from_file
DataBase, IdTag -> Tag: database.get_tag_from_id_tag(id_tag) | get_mut_tag_from_id_tag
Database, WordAddress -> Tag: database.get_tag_from_word_address(word_address) | get_mut_tag_from_word_address
Database, WordAddress, nb_words -> Vec<Tag>: database.get_tags_from_word_address_area(word_address, nb_words)
Tag -> String: format!(tag)
Tag, WordAddress -> bool: tag.contains_word_address_area(word_address)
Database, WordAddress -> <type>: database.get_<type>_from_word_address
Database, <type>, WordAddress -> update_database: database.set_<type>_to_word_address
Database, IdTag -> <type>: database.get_<type>_from_id_tag(id_tag)
Database, <type>, IdTag -> update_database: database.set_<type>_to_id_tag
