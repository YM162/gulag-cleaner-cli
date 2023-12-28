//WARNING: I need to add comments to all of the files in this folder. I will do it ASAP. I'm sorry for the inconvenience.


pub mod method;

pub mod page_type;

use lopdf::{Dictionary, Document, Object, ObjectId};

use std::{collections::HashSet, error::Error};



pub fn clean_pdf(data: Vec<u8>,force_naive: u8) -> (Vec<u8>,u8) {

    //Load the PDF into a Document
    let mut doc = Document::load_mem(&data).unwrap();
    let pages = doc.get_pages();

    //We first need to determine what method we're using, either "Wuolah", "StuDocu" or "Wuolah naive". We keep it like this to allow for future methods if needed.

    let method = method::Method::new(&doc, force_naive);

    //Each method should mark pages for deletion in to_delete and modify the contents of the pages.
    
    let (to_delete,method_code) = match method {
        method::Method::Wuolah(content_list, to_delete) => {
            println!("Using Wuolah method");
            let new_contents: Vec<Vec<(u32, u16)>> = content_list
                .iter()
                .enumerate()
                .map(|(i, x)| {
                    let pares = if x == content_list.last().unwrap() {
                        find_iobj_pairs(x, &content_list[i - 1])
                    } else {
                        let check_if_00 = find_iobj_pairs(x, &content_list[i + 1]);
                        if check_if_00 != (0,0) {
                            check_if_00
                        } else {
                            find_iobj_pairs(x, &content_list[i - 1])
                        }
                    };

                    x[(pares.0) - 2..=(pares.1) + 3].to_vec()
                })
                .collect();

            let vector: Vec<(&u32, &(u32, u16))> = pages
                .iter()
                .filter(|x| doc.get_page_contents(*x.1).len() > 1)
                .collect();
            for (i, page) in vector.iter().enumerate() {
                let mutable_page = doc.get_object_mut(*page.1).unwrap().as_dict_mut().unwrap();
                let contents_objects: Vec<Object> = new_contents[i]
                    .iter()
                    .map(|x| Object::Reference(*x))
                    .collect();

                mutable_page.set(*b"Contents", lopdf::Object::Array(contents_objects));

                mutable_page.set("Annots", Object::Array(vec![]));
                let mediabox = mutable_page.get(b"MediaBox").unwrap().as_array().unwrap();
                
                let height_offset = match mediabox[1].as_f32() {
                    Ok(h) => h,
                    _ => mediabox[1].as_i64().unwrap() as f32,
                };
                let width_offset = match mediabox[0].as_f32() {
                    Ok(h) => h,
                    _ => mediabox[0].as_i64().unwrap() as f32,
                };

                let height = match mediabox[3].as_f32() {
                    Ok(h) => h,
                    _ => mediabox[3].as_i64().unwrap() as f32,
                };
                let width = match mediabox[2].as_f32() {
                    Ok(h) => h,
                    _ => mediabox[2].as_i64().unwrap() as f32,
                };

                for _box in ["MediaBox", "ArtBox", "TrimBox", "CropBox", "BleedBox"] {
                    mutable_page.set(
                        _box,
                        Object::Array(vec![
                            Object::Real(0.0),
                            Object::Real(0.0),
                            Object::Real(width - width_offset),
                            Object::Real(height - height_offset),
                        ]),
                    );
                };
            }


            (to_delete,0)
        }

        method::Method::StuDocu(content_list) => {
            println!("Using StuDocu method");
            let new_contents: Vec<Vec<(u32, u16)>> = content_list
                .iter()
                .skip(1)
                .map(|x| {
                    vec![x[1]]
                })
                .collect();

            let vector: Vec<(&u32, &(u32, u16))> = pages
                .iter()
                .filter(|x| *x.0 != 1)
                .collect();
            for (i, page) in vector.iter().enumerate() {
                let mutable_page = doc.get_object_mut(*page.1).unwrap().as_dict_mut().unwrap();
                let contents_objects: Vec<Object> = new_contents[i]
                    .iter()
                    .map(|x| Object::Reference(*x))
                    .collect();

                mutable_page.set(*b"Contents", lopdf::Object::Array(contents_objects));

                mutable_page.set("Annots", Object::Array(vec![]));
            }
            (vec![1],1)
        }

        method::Method::Naive => {
            println!("Using naive method");
            let mut to_delete = Vec::new();

            for page in &pages {
                let page_type = page_type::PageType::get_page_type(&doc, page.1).unwrap_or_default();
                let mutable_page = doc.get_object_mut(*page.1).unwrap().as_dict_mut().unwrap();

                let mediabox = mutable_page.get(b"MediaBox").unwrap().as_array().unwrap();
                let height_offset = match mediabox[1].as_f32() {
                    Ok(h) => h,
                    _ => mediabox[1].as_i64().unwrap() as f32,
                };
                let width_offset = match mediabox[0].as_f32() {
                    Ok(h) => h,
                    _ => mediabox[0].as_i64().unwrap() as f32,
                };

                let height = match mediabox[3].as_f32() {
                    Ok(h) => h,
                    _ => mediabox[3].as_i64().unwrap() as f32,
                };
                let width = match mediabox[2].as_f32() {
                    Ok(h) => h,
                    _ => mediabox[2].as_i64().unwrap() as f32,
                };
                
                match page_type {
                    page_type::PageType::FullPageAds => to_delete.push(*page.0),
                    page_type::PageType::Idk => to_delete.push(*page.0),
                    page_type::PageType::BannerAds => {
                        //1.141
                        let scale = 1.124;
                        for _box in ["MediaBox", "ArtBox", "TrimBox", "CropBox", "BleedBox"] {
                            mutable_page.set(
                                _box,
                                Object::Array(vec![
                                    Object::Real(0.164 * (width-width_offset) + width_offset*scale),
                                    Object::Real(0.031 * (height-height_offset) + height_offset*scale),
                                    Object::Real(0.978 * (width-width_offset) * scale + width_offset*scale),
                                    Object::Real(0.865 * (height-height_offset) * scale + height_offset*scale),
                                ]),
                            );
                        };

                        let mut contents = doc.get_page_content(*page.1).unwrap();
                        let mut new_contents = Vec::new();
                        let c_prepend = "q\n1.124 0 0 1.124 0 0 cm\n".as_bytes();
                        let c_append = "Q".as_bytes();

                        new_contents.extend_from_slice(c_prepend);
                        new_contents.append(&mut contents);
                        new_contents.extend_from_slice(c_append);

                        doc.change_page_content(*page.1, new_contents).unwrap()
                    }
                    page_type::PageType::Watermark => {
                        for _box in ["MediaBox", "ArtBox", "TrimBox", "CropBox", "BleedBox"] {
                            mutable_page.set(
                                _box,
                                Object::Array(vec![
                                    Object::Real(0.015 * (width-width_offset) + width_offset),
                                    Object::Real(0.05 * (height-height_offset) + height_offset),
                                    Object::Real(0.95 * (width-width_offset) + width_offset),
                                    Object::Real(0.98 * (height-height_offset) + height_offset),
                                ]),
                            );
                        };
                    }
                }
            }

            for page in &pages {
                // remove the logo
                let _ = remove_logo(&mut doc, page.1);

                
                // remove the annotations
                let mutable_page = doc.get_object_mut(*page.1).unwrap().as_dict_mut().unwrap();
                mutable_page.set("Annots", Object::Array(vec![]));
                
            }

            (to_delete,2)
        }
    };

    //Delete the pages that we've marked for deletion.
    for (offset, page) in to_delete.into_iter().enumerate() {
        doc.delete_pages(&[page - offset as u32]);
    }
    //Save the document.
    let mut return_stream = Vec::new();
    doc.save_to(&mut return_stream).unwrap();

    // Should we still return the method_code now that we are going multi-language? I will leave it not returned for now.
    //return_stream.push(method_code);
    (return_stream,method_code)
    //doc.save_to("test.pdf").unwrap();
}

fn find_iobj_pairs(first_page: &[(u32, u16)], second_page: &[(u32, u16)]) -> (usize, usize) {
    let unique_first_page: HashSet<&(u32, u16)> = first_page.iter().collect();
    let unique_second_page: HashSet<&(u32, u16)> = second_page.iter().collect();

    let c: Vec<&&(u32, u16)> = unique_first_page
        .intersection(&unique_second_page)
        .collect();
    if c.len() != 2 {
        return (0, 0);
    }
    let first_index = first_page.iter().position(|&r| r == **c[0]).unwrap();
    let second_index = first_page.iter().position(|&r| r == **c[1]).unwrap();

    if first_index < second_index {
        (first_index, second_index)
    } else {
        (second_index, first_index)
    }
}

fn get_xobjs<'a>(doc: &'a Document, page: &ObjectId) -> Result<&'a Dictionary, Box<dyn Error>> {
    
    let resource = doc.get_page_resources(*page);
    let resource_dict;
    if resource.1.is_empty() {
        resource_dict = resource.0.unwrap();
    } else {
        resource_dict = doc.get_object(resource.1[0])?.as_dict()?;
    }
    
    let xobjs = resource_dict
        .get(b"XObject")?
        .as_dict()?;
    Ok(xobjs)
}

fn get_objdict<'a>(
    doc: &'a Document,
    obj: (&Vec<u8>, &Object),
) -> Result<&'a Dictionary, Box<dyn Error>> {
    let objdict = &doc
        .get_object(obj.1.as_reference().unwrap())?
        .as_stream()?
        .dict;

    Ok(objdict)
}

fn get_images(doc: &Document, xobjs: &Dictionary) -> Result<Vec<(i64, i64)>, Box<dyn Error>> {
    let mut images = Vec::new();

    for obj in xobjs {
        let objectdict = get_objdict(doc, obj)?;

        let subtype = objectdict.get(b"Subtype").unwrap().as_name().unwrap();
        let sub_s = String::from_utf8_lossy(subtype);
        
        if sub_s.starts_with("Image") {
            images.push((
                objectdict.get(b"Height").unwrap().as_i64().unwrap(),
                objectdict.get(b"Width").unwrap().as_i64().unwrap(),
            ));
        }
    }
    
    Ok(images)
}

fn remove_logo(doc: &mut Document, page: &ObjectId) -> Result<(), Box<dyn Error>> {
    let xobjs = get_xobjs(doc, page)?.clone();
    let images = get_images(doc, &xobjs)?;
    
    let has_logo = !page_type::LOGO_DIMS
        .iter()
        .collect::<HashSet<_>>()
        .intersection(&images.iter().collect::<HashSet<_>>())
        .collect::<Vec<_>>()
        .is_empty();

    if has_logo {
        for obj in &xobjs {
            let objectdict = get_objdict(doc, obj)?;
            
            let subtype = objectdict.get(b"Subtype")?.as_name()?;
            
            let sub_s = String::from_utf8_lossy(subtype);
            if sub_s.starts_with("Image")
                && page_type::LOGO_DIMS.contains(&(
                    objectdict.get(b"Height")?.as_i64()?,
                    objectdict.get(b"Width")?.as_i64()?,
                ))
            {
                let mutable_page = &mut doc
                    .get_object_mut(obj.1.as_reference()?)?
                    .as_stream_mut()?
                    .dict;
                mutable_page.set(*b"Height", 0);
            }
            {
        }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::clean_pdf;
    #[test]
    fn test_pdf() {
        //Load some pdf bytes and clean it
        let data = std::fs::read("../test.pdf").unwrap();
        let (clean_pdf,_) = clean_pdf(data,0);
        std::fs::write("../test_clean.pdf", clean_pdf).unwrap();
    }
}