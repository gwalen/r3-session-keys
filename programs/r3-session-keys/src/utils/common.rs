pub fn read_array_element(array: &[u8], index: usize, element_size: usize) -> &[u8] {
    let start = index * element_size;
    let end = start + element_size;
    // array[start..end].to_vec()
    &array[start..end]
}

pub fn write_array_element(array: &mut [u8], index: usize, element: &[u8]) {
    let start = index * element.len();
    let end = start + element.len();
    array[start..end].copy_from_slice(element);
}