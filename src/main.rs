
use image::{io::Reader as ImageReader,Rgb,RgbImage};
use rand::prelude::*;
use std::thread::sleep;
use std::{io::Write, vec};
use std::env;
use std::fs::File;
use bitstream_io::{BitWrite, BitWriter};
use std::time::Duration;






fn genrate_huffman_code(counts:&Vec<u32>) -> Vec<(u8,u8)> {

    let mut codes:Vec<(u8,u8)> = vec![(0,0);counts.len()];



    let mut map:Vec<(u32,Vec<usize>)> = Vec::with_capacity(counts.len());
    for i in 0..counts.len() {
        map.push((counts[i],vec![i]));

    }

    map.sort_by(|a, b| b.0.cmp(&a.0));

    while map.len() > 1 {


        let (count_1,indices_1) = map.pop().unwrap();
        let (count_2,indices_2) = map.pop().unwrap();


        for i in indices_1.iter() {
            codes[*i].0 += 1;
        }


        for i in indices_2.iter() {
            codes[*i].1 = codes[*i].1 | (1 << codes[*i].0);
            codes[*i].0 += 1;

        }

        let count = count_1 + count_2;
        let indices = [indices_1,indices_2].concat();

        let mut index = map.len();

        while index > 0 && map[index - 1].0 < count {
            index -= 1;
        }

    
        map.insert(index, (count,indices));
    }

    codes
}



fn main() {

    let args: Vec<String> = env::args().collect();

    let mut k:usize = 0;

    let mut image_path = String::new();

    let mut output_path = String::new();


    for i in 1..args.len() {
        if args[i] == "--colors" {
            k = args[i + 1].parse().unwrap();
        }
        if args[i] == "--path" {
            image_path = args[i + 1].to_string();
        }
        if args[i] == "--output" {
            output_path = args[i + 1].to_string();
        }
    }


    let mut output_file = File::create(output_path).unwrap();
   

    


    let img = ImageReader::open(image_path).unwrap().decode().unwrap().to_rgb8();

    let mut quantized_values:Vec<(u8,u8,u8)> = Vec::with_capacity(k);
    let mut rng = rand::thread_rng();
    for _ in 0..k {
        let r:u8 = rng.gen();
        let g:u8 = rng.gen();
        let b:u8 = rng.gen();
        quantized_values.push((r,g,b));
    }

    let (width, height) = img.dimensions();

    let mut clusters = vec![0 as usize; width as usize * height as usize];


    let mut total: Vec<(i32, i32, i32)> = vec![(0,0,0);k];

    let mut count = vec![0; k];
    
    let mut diff:f32 = 1.0;

    while diff > 0.05 {
        diff = 0.0;

        total.fill((0,0,0));

        count.fill(0);

  
  

        for y in 0..height {
            for x in 0..width {
           
                let pixel = img.get_pixel(x, y);
                let mut min = i32::MAX;
      
    
                for i in 0..k {
                    let p = quantized_values[i as usize];
                    let distance = (pixel[0] as i32 - p.0 as i32).pow(2) + (pixel[1] as i32 - p.1 as i32).pow(2) + (pixel[2] as i32 - p.2 as i32).pow(2);
                    if distance < min {
                        min = distance;
                        clusters[(y  * width  + x) as usize] = i;
                    }
                }

                let t= &mut total[clusters[(y  * width  + x) as usize]];
      
                t.0 += pixel[0] as i32;
                t.1 += pixel[1] as i32;
                t.2 += pixel[2] as i32;
    
                count[clusters[(y  * width  + x) as usize] as usize] += 1;
            }    

            
        }


        for i in 0..k{
            if count[i] == 0 {continue;}
            let t = &mut total[i];

            t.0 /= count[i];
            t.1 /= count[i];
            t.2 /= count[i];

            let p = & mut quantized_values[i];


            
            diff = diff.max(
                (t.0 as i32 - p.0 as i32).abs() as f32 / p.0 as f32
            );

            diff = diff.max(
                (t.1 as i32 - p.1 as i32).abs() as f32 / p.1 as f32
            );
      

            diff = diff.max(
                (t.2 as i32 - p.2 as i32).abs() as f32 / p.2 as f32
            );

            p.0 = t.0 as u8;
            p.1 = t.1 as u8;
            p.2 = t.2 as u8;



            
        }


    
    }


    let mut total_cluster  = vec![0 as u32;k];



    for cluster in clusters.iter() {
        
        total_cluster[*cluster] += 1;
    }


    let codes = genrate_huffman_code(&total_cluster);


  



    let mut writer = BitWriter::endian(output_file, bitstream_io::BigEndian);



    writer.write(16,width).unwrap();
    writer.write(16,height).unwrap();






    writer.write(8, k as u8).unwrap();






    for i in 0..k {
        let (length,code) = codes[i];
        writer.write(4, length).unwrap();
        writer.write(length as u32, code).unwrap();
        let (r,g,b) = quantized_values[i];
        writer.write(8, r).unwrap();
        writer.write(8, g).unwrap();
        writer.write(8, b).unwrap();


    }


    

    for cluster in clusters.iter() {
        let (length,code) = codes[*cluster];
        writer.write(length as u32, code).unwrap();

    }


    
    writer.write(20, 0);

    














}