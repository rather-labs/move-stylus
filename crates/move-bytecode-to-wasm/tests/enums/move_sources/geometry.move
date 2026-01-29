// Copyright (c) Mysten Labs, Inc.
// Copyright (c) 2025 Rather Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

module 0x00::enums_geometry;

public enum Shape has copy, drop {
    Triangle {
        base: u64,
        height: u64
    },
     Square {
        side: u64,
    }
}

const WRONG_SHAPE: u64 = 1;

fun create_square(side: u64): Shape {
    Shape::Square { side }
}

fun create_triangle(base: u64, height: u64): Shape {
    Shape::Triangle { base, height }
}

fun get_area(shape: Shape): u64 {
    match (shape) {
        Shape::Square { side } => side * side,
        Shape::Triangle { base, height } => base * height / 2,
    }
}

fun set_triangle_dimensions(shape: &mut Shape, new_base: u64, new_height: u64) {
    match (shape) {
        Shape::Triangle {  base,  height } => {
            *base = new_base;
            *height = new_height;
        },
        _ =>  abort(WRONG_SHAPE),
    }
}

fun set_square_side(shape: &mut Shape, new_side: u64) {
    match (shape) {
        Shape::Square { side } => {
            *side = new_side;
        },
        _ =>  abort(WRONG_SHAPE),
    }
}

fun get_square_side(shape: &Shape): u64 {
    match (shape) {
        Shape::Square { side } => *side,
        _ => abort(WRONG_SHAPE),
    }
}

fun get_triangle_size(shape: &Shape): (u64, u64) {
    match (shape) {
        Shape::Triangle { base, height } => (*base, *height),
        _ => abort(WRONG_SHAPE),
    }
}

entry fun test_square(side: u64): (u64, u64) {
    let square = create_square(side);
    let side0 = get_square_side(&square);
    let a0 = get_area(square);
    (side0, a0)
}

entry fun test_mutate_square(side: u64): (u64, u64, u64, u64) {
    let mut square = create_square(side);
    let a0 = get_area(square);
    let side0 = get_square_side(&square);
    set_square_side(&mut square, side + 1);
    let a1 = get_area(square);
    let side1 = get_square_side(&square);
    (side0, a0, side1, a1)
}

entry fun test_triangle(base: u64, height: u64): (u64, u64, u64) {
    let triangle = create_triangle(base, height);
    let a0 = get_area(triangle);
    let (base0, height0) = get_triangle_size(&triangle);
    (base0, height0, a0)
}

entry fun test_mutate_triangle(base: u64, height: u64): (u64, u64, u64, u64, u64, u64) {
    let mut triangle = create_triangle(base, height);
    let a0 = get_area(triangle);
    let (base0, height0) = get_triangle_size(&triangle);
    set_triangle_dimensions(&mut triangle, base + 1, height + 1);
    let a1 = get_area(triangle);
    let (base1, height1) = get_triangle_size(&triangle);
    (base0, height0, a0, base1, height1, a1)
}

entry fun test_vector_of_shapes_1(a: u64, b: u64): (u64, u64, u64) {
    let ve = vector[Shape::Square { side: a }, Shape::Triangle { base: a, height: b }];
    let len = vector::length(&ve);
    let a0 = get_area(ve[0]);
    let a1 = get_area(ve[1]);
    (len, a0, a1)
}

entry fun test_vector_of_shapes_2(a: u64, b: u64): (u64, u64, u64) {
    let mut ve = vector[Shape::Square { side: a }, Shape::Triangle { base: 2, height: 3 }];
    vector::pop_back(&mut ve);
    vector::push_back(&mut ve, Shape::Triangle { base: a, height: b });
    vector::swap(&mut ve, 0, 1);
    let len = vector::length(&ve);
    let a0 = get_area(ve[0]);
    let a1 = get_area(ve[1]);
    (len, a0, a1)
}
