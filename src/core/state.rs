use enum_iterator::Sequence;


#[derive(Debug, Sequence)]
pub enum PacketState {

    Received,
    Prepared,
    PostPrepared,

}


