use clap::Args;

use super::ConfigBuilder;

#[derive(Args, Clone, Debug, Default)]
pub struct QuerySubCommand {
    #[arg(short, long)]
    pub relay: Option<String>,
    #[arg(short, long)]
    pub authors: Option<String>,
    #[arg(short, long)]
    pub ids: Option<String>,
    #[arg(short, long)]
    pub limit: Option<i32>,
    #[arg(long)]
    pub generic: Option<Vec<String>>,
    #[arg(short = 't', long)]
    pub hashtag: Option<String>,
    #[arg(short, long)]
    pub mentions: Option<String>,
    #[arg(short = 'e', long)]
    pub references: Option<String>,
    #[arg(short, long)]
    pub kinds: Option<String>,
    #[arg(short, long)]
    pub search: Option<Vec<String>>,
}

impl QuerySubCommand {
    pub fn into_config_builder(self) -> ConfigBuilder {
        let mut builder = ConfigBuilder::default();

        if let Some(relay) = self.relay {
            builder = builder.host(&relay);
        }
        if let Some(authors) = self.authors {
            builder = builder.authors(&authors);
        }
        if let Some(ids) = self.ids {
            builder = builder.ids(&ids);
        }
        if let Some(limit) = self.limit {
            builder = builder.limit(limit);
        }
        if let Some(generic) = self.generic {
            if generic.len() == 2 {
                builder = builder.generic(&generic[0], &generic[1]);
            }
        }
        if let Some(hashtag) = self.hashtag {
            builder = builder.hashtag(&hashtag);
        }
        if let Some(mentions) = self.mentions {
            builder = builder.mentions(&mentions);
        }
        if let Some(references) = self.references {
            builder = builder.references(&references);
        }
        if let Some(kinds) = self.kinds {
            builder = builder.kinds(&kinds);
        }
        if let Some(search) = self.search {
            if search.len() == 2 {
                builder = builder.search(&search[0], &search[1]);
            }
        }

        builder
    }
}
