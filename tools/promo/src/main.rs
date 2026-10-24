use diggerchain_promo::{Campaign, CampaignMedia};
use fframes::cli::clap;
use fframes::{EncoderOptions, RenderOptions, StaticMediaProvider, cli};
use fframes_skia_renderer::{SkiaFFramesRenderer, SkiaPipelineConfig, vulkan::SkiaVulkanCtx};
use std::process::ExitCode;

#[derive(Debug, clap::Args)]
struct VideoArgs {
    #[arg(long, global = true)]
    portrait: bool,
}

fn render<const W: usize, const H: usize>(
    args: cli::Cli<VideoArgs>,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let media = CampaignMedia::prepare()?;
    let video = Campaign::<W, H>::new()?;
    let gpu = SkiaVulkanCtx::new(W, H)?;
    Ok(cli::new(
        &video,
        RenderOptions {
            media: Some(&media),
            default_font: "Libre Baskerville",
            video_encoder_options: EncoderOptions {
                preferred_encoder: Some("libx264"),
                codec_params: Some(&[("preset", "fast"), ("crf", "18")]),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .args(args)
    .backend(SkiaFFramesRenderer::new_vulkan(
        &gpu,
        SkiaPipelineConfig::default(),
    )?)
    .preview(fframes_native_player::cli_preview)
    .run())
}
fn main() -> ExitCode {
    let args = cli::parse::<VideoArgs>();
    let result = if args.app.portrait {
        render::<1080, 1920>(args)
    } else {
        render::<1920, 1080>(args)
    };
    match result {
        Ok(code) => code,
        Err(error) => {
            eprintln!("DiggerChain promo: {error}");
            ExitCode::FAILURE
        }
    }
}
