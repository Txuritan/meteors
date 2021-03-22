// response of /api/index, or 403
#[derive(Clone, PartialEq, prost::Message)]
pub struct IndexResponse {}

#[derive(Clone, PartialEq, prost::Message)]
pub struct StoryRequest {}

#[derive(Clone, PartialEq, prost::Message)]
pub struct StoryResponse {}
