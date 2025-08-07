use super::IType;

#[derive(Hash)]
pub struct IRef;

impl IType for IRef {}

#[derive(Hash)]
pub struct IMutRef;

impl IType for IMutRef {}
