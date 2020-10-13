use crate::context::Context;
use crate::geometry::{SimpleMesh, ToSimpleMesh, ToSimpleMeshWithMaterial};
use std::error::Error;
use std::fs::OpenOptions;
use std::path::PathBuf;

use clap::Clap;

/// A toy for rendering 3D objects in the command line
#[derive(Clap)]
#[clap(version, author, about)]
pub struct Opts {
    /// Sets the input file to render
    #[clap()]
    pub models: Vec<PathBuf>,

    /// Flags the rasterizer to render without color
    #[clap(short = 'c')]
    pub color_less: bool,

    /// Sets the object's static X rotation (in radians)
    #[clap(short = 'x', long = "yaw", default_value = "0.0")]
    pub x: f32,

    /// Sets the object's static Y rotation (in radians)
    #[clap(short = 'y', long = "pitch", default_value = "0.0")]
    pub y: f32,

    /// Sets the object's static Z rotation (in radians)
    #[clap(short = 'z', long = "roll", default_value = "0.0")]
    pub z: f32,

    /// Speed of rotation
    #[clap(short = 's', default_value = "1.0")]
    pub speed: f32,

    #[clap(subcommand)]
    pub subcmd: SubCommand,
}

#[derive(Clap, PartialEq)]
pub enum SubCommand {
    Interactive,
    /// Generates a colorless terminal output as lines of text
    Image {
        /// Generates a portable JS based render of your object for the web
        #[clap(short = 'j', default_value = "false")]
        webify: u16,

        /// Sets the width of the image to generate
        #[clap(short = 'w')]
        width: u16,

        /// Sets the height of the image to generate
        #[clap(short = 'h')]
        height: Option<u16>,
    },
}

impl Opts {
    fn to_meshes(models: Vec<tobj::Model>, materials: Vec<tobj::Material>) -> Vec<SimpleMesh> {
        let mut meshes: Vec<SimpleMesh> = vec![];
        for model in models {
            meshes.push(model.mesh.to_simple_mesh_with_materials(&materials));
        }
        meshes
    }

    pub fn match_meshes(&self) -> Result<Vec<SimpleMesh>, Box<dyn Error>> {
        let mut mesh_queue: Vec<SimpleMesh> = vec![];
        for path in &self.models {
            let error = |s: &str, e: &str| -> Result<Vec<SimpleMesh>, Box<dyn Error>> {
                Err(format!("filename: [{:?}] couldn't load, {}. {}", path, s, e).into())
            };
            // Fill list with file inputs (Splits for spaces -> multiple files)
            let meshes = match path.extension() {
                None => error("couldn't determine filename extension", ""),
                Some(ext) => match ext.to_str() {
                    None => error("couldn't parse filename extension", ""),
                    Some(extstr) => match &*extstr.to_lowercase() {
                        "obj" => match tobj::load_obj(&path, true) {
                            Err(e) => error("tobj couldnt load/parse OBJ", &e.to_string()),
                            Ok(present) => Ok(Self::to_meshes(present.0, present.1)),
                        },
                        "stl" => match OpenOptions::new().read(true).open(&path) {
                            Err(e) => error("STL load failed", &e.to_string()),
                            Ok(mut file) => match stl_io::read_stl(&mut file) {
                                Err(e) => error("stl_io couldnt parse STL", &e.to_string()),
                                Ok(stlio_mesh) => Ok(vec![stlio_mesh.to_simple_mesh()]),
                            },
                        },
                        _ => error("unknown filename extension", ""),
                    },
                },
            };
            mesh_queue.append(&mut meshes.unwrap());
        }
        Ok(mesh_queue)
    }

    pub fn match_turntable(&self) -> Result<(f32, f32, f32, f32), Box<dyn Error>> {
        let mut turntable = (0.0, 0.0, 0.0, 0.0);
        turntable.0 = self.x;
        turntable.1 = self.y;
        turntable.2 = self.z;
        turntable.3 = self.speed;
        turntable.1 += std::f32::consts::PI; // All models for some reason are backwards, this fixes that
        Ok(turntable)
    }

    pub fn match_dimensions(&self, context: &mut Context) -> Result<(), Box<dyn Error>> {
        /*
        if let Some(x) = matches.value_of("width") {
            context.width = x.parse()?;
            if let Some(y) = matches.value_of("height") {
                context.height = y.parse()?;
            } else {
                context.height = context.width;
            }
        }
        */
        Ok(())
    }
}
