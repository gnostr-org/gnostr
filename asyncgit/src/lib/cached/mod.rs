//! cached lookups:
//! parts of the sync api that might take longer
//! to compute but change seldom so doing them async might be overkill

pub use gitui_asyncgit::cached::BranchName;
