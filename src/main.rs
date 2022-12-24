use std::{fs::File, io::BufWriter};

use brickadia::read::SaveReader;
use clap::Parser;
use part::convert_brick;
use rbx_dom_weak::{InstanceBuilder, WeakDom};

pub mod cframe;
mod part;

#[derive(Parser)]
#[command(
    author = "voximity",
    version = "1.0",
    about = "Convert between Brickadia .brs files and Roblox .rbxm files"
)]
struct Cli {
    input: String,
    #[arg(short = 'o')]
    output: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    let out = cli.output.unwrap_or_else(|| cli.input.clone() + ".rbxm");

    let save = SaveReader::new(File::open(cli.input.as_str()).unwrap())
        .unwrap()
        .read_all_skip_preview()
        .unwrap();

    let mut model = InstanceBuilder::new("Model");
    model.set_name(cli.input.as_str());

    model.add_child(
        InstanceBuilder::new("Script")
            .with_name("brs2rbxl")
            .with_property(
                "Source",
                format!(
                    "print'\"{}\"'print'Saved by {}'print''print'Exported from Brickadia with rbxl-brs'",
                    save.header1.description, save.header1.author.name
                ),
            ),
    );

    for brick in save.bricks.iter() {
        let asset = save.header2.brick_assets[brick.asset_name_index as usize].as_str();

        let name = format!(
            "{} (dir {}, rot {})",
            asset, brick.direction as u8, brick.rotation as u8
        );

        match convert_brick(brick, &save) {
            Some(instances) => {
                if instances.len() == 1 {
                    let child = instances.into_iter().next().unwrap();
                    model.add_child(child.with_name(name));
                } else {
                    let mut group = InstanceBuilder::new("Model").with_name(name);
                    instances.into_iter().for_each(|i| group.add_child(i));
                    model.add_child(group);
                }
            }
            None => println!("Unimplemented brick converter for asset {}", asset),
        };
    }

    let dom = WeakDom::new(model);

    let writer = BufWriter::new(File::create(out).unwrap());
    rbx_binary::to_writer(writer, &dom, &[dom.root_ref()]).unwrap();
}
