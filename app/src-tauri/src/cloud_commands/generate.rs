use super::*;
pub(super) use crate::cloud::generation_workflow::audio_type;
struct NativeProviders<'a>(&'a AppState);
impl crate::cloud::generation_workflow::Providers for NativeProviders<'_> {
 fn provider(&self,id:&str)->Result<(ProviderConfig,String)>{Ok((config(self.0,id)?,credentials::read_for_service(id)?))}
}
pub(super) async fn step(state:&AppState,root:&Path,j:&mut Value)->Result<()> {
 crate::cloud::generation_workflow::step(&NativeProviders(state),root,j).await
}
