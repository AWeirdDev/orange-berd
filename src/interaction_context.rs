use std::ops::Deref;

use anyhow::{ Context, Result };
use twilight_http::{ Client as HttpClient, client::InteractionClient };
use twilight_model::{
    application::interaction::Interaction,
    channel::message::MessageFlags,
    http::interaction::{ InteractionResponse, InteractionResponseData, InteractionResponseType },
};
use twilight_util::builder::InteractionResponseDataBuilder;

pub trait ToInteractionResponseData {
    fn into_ird(self) -> InteractionResponseData;
}

impl ToInteractionResponseData for InteractionResponseData {
    fn into_ird(self) -> InteractionResponseData {
        self
    }
}

impl ToInteractionResponseData for &str {
    fn into_ird(self) -> InteractionResponseData {
        InteractionResponseDataBuilder::new().content(self).build()
    }
}

pub(crate) struct CommandInteractionContext<'a> {
    pub(crate) client: InteractionClient<'a>,
    pub(crate) interaction: &'a Interaction,
}

impl<'a> CommandInteractionContext<'a> {
    pub(crate) fn new(http: &'a HttpClient, interaction: &'a Interaction) -> Self {
        Self { client: http.interaction(interaction.application_id), interaction }
    }

    pub(crate) async fn defer(&self, ephemeral: bool) -> Result<()> {
        self.client
            .create_response(
                self.interaction.id,
                &self.interaction.token,
                &(InteractionResponse {
                    kind: InteractionResponseType::DeferredChannelMessageWithSource,
                    data: Some(
                        InteractionResponseDataBuilder::new()
                            .flags({
                                if ephemeral {
                                    MessageFlags::EPHEMERAL
                                } else {
                                    MessageFlags::empty()
                                }
                            })
                            .build()
                    ),
                })
            ).await
            .context("Failed to defer()")?;

        Ok(())
    }

    pub(crate) async fn respond<K: ToInteractionResponseData>(&self, data: K) -> Result<()> {
        self.client.create_response(
            self.interaction.id,
            &self.interaction.token,
            &(InteractionResponse {
                kind: InteractionResponseType::ChannelMessageWithSource,
                data: Some(data.into_ird()),
            })
        ).await?;

        Ok(())
    }
}

impl<'a> Deref for CommandInteractionContext<'a> {
    type Target = InteractionClient<'a>;
    fn deref(&self) -> &Self::Target {
        &self.client
    }
}
