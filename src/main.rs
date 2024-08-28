mod color;
use std::{io::Write, time::{Duration, SystemTime}};

use anyhow::{Context, Result};
use crossterm::{terminal, cursor, queue, style};
use color::{Color, HslColor};

use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256PlusPlus;
use rand::{Rng, RngCore};

#[derive(Clone, Copy)]
struct Glyph {
    character: char,
    color: Color
}

impl Glyph {
    fn new(character: char, color: Color) -> Self{
        Self{
            character,
            color
        }
    }

    fn fade_color(&mut self){
        let hsl = self.color.as_hsl();
        self.color = HslColor::new(hsl.h, hsl.s*0.8, hsl.l*0.8).into();
    }
    fn render<W: Write> (&self, out: &mut W)-> Result<()>{
        queue!(
            out,
            style::SetBackgroundColor(style::Color::Rgb {
                r: 0, g: 0, b: 0 })
        )?;
        queue!(
            out,
            style::SetForegroundColor(style::Color::Rgb
                { r: self.color.r, g: self.color.g, b:self.color.b })
        )?;
        queue!(out, style::Print(self.character.to_string()))?;
        out.write(self.character.to_string().as_bytes()).context("write glyphs")?;
        Ok(())
    }

    fn empty()-> Self {
        Self{
            character: ' ',
            color: Color::from_rgba(0,0,0,0)
        }
    }

    fn new_random<R: Rng>(rand: &mut R, color: Color)-> Self{
        let characters = "ﾊﾐﾋｰｳｼﾅﾓﾆｻﾜﾂｵﾘｱﾎﾃﾏｹﾒｴｶｷﾑﾕﾗｾﾈｽﾀﾇﾍｦｲｸｺｿﾁﾄﾉﾌﾔﾖﾙﾚﾛﾝ012345789Z:.\"=*+-<>¦╌ç";

        Self{
            character: characters.chars().nth
                (rand.gen_range(0..characters.chars().count()))
                .unwrap()
            ,
            color
        }
    }
}

#[derive(Clone)]
struct Column {
    height: u16,
    base_color: Color,
    glyphs: Vec<Glyph>,
    active_index: usize,
}

impl Column {
    fn new(height: u16, base_color: Color)-> Self{
        let glyphs = vec![Glyph::empty(); height as usize];

        Self{
            height,
            base_color,
            glyphs,
            active_index:0
        }
    }

    fn empty(height: u16) ->Self{
        Self{
            height,
            base_color: Color::from_rgba(0,0,0,255),
            glyphs: vec![Glyph::empty(); height as usize],
            active_index: 0
        }
    }
    fn render<W: Write>(&self, out: &mut W, y: u16)-> Result<()>{
        self.glyphs[y as usize].render(out)?;
        Ok(())
    }
    fn step<R: Rng>(&mut self, rand: &mut R){

        if self.active_index==0 && rand.gen::<f32>() >0.1{
            return;
        }

        for glyph in &mut self.glyphs {
            glyph.fade_color();
        }

        self.glyphs[self.active_index] = Glyph::new_random(rand,self.base_color);
        self.active_index+=1;

        if self.active_index >=self.height as usize{
            self.active_index = 0
        }
    }
}

struct MatrixWaterfall{
    height: u16,
    width: u16,
    base_color: Color,
    columns: Vec<Column>
}

impl MatrixWaterfall {
    fn new(w: u16,h: u16,col:Color)-> Self{
        Self{
            width: w,
            height: h,
            base_color: col,
            columns: vec![Column::new(h,col); w as usize]
        }
    }

    fn render<W: Write>(&self, out: &mut W)-> Result<()>{

        queue!(out, cursor::Hide)?;
        queue!(out, cursor::MoveTo(0,0));

        for y in 0..self.height{
            for column in &self.columns{
                column.render(out,y)?;
            }
        }
        queue!(out, cursor::Show)?;
        out.flush().context("flush output")?;
        Ok(())
    }


    fn step<R: RngCore>(&mut self, rand: &mut R){

        for column in &mut self.columns {
            column.step(rand);
        }
    }
}

fn main() -> Result<()> {

    let (width, height) = terminal::size().context("determine terminal size")?;
    let base_color = Color::from_rgba(0,255,43,255);
    let mut waterfall = MatrixWaterfall::new(width, height, base_color);

    let mut stdout = std::io::stdout();

    let seed = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).expect("some time").as_micros() as u64;
    let mut random = Xoshiro256PlusPlus::seed_from_u64(seed);

    loop{
        waterfall.render(&mut stdout)?;
        waterfall.step(&mut random);
        std::thread::sleep(Duration::from_millis(100));
    }
}
