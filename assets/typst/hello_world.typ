#let wave_gen(func, frequency, amplitude, time, resolution) = {
  let result = ()
  let inv_res = 1.0 / resolution
  for i in range(0, resolution + 1) {
    result.push((
      100% * i * inv_res,
      (func((time + calc.pi * float(i) / resolution) * frequency) * amplitude) - 50%,
    ))
  }
  result
}

#let main(animate) = {
  let t = animate * 2
  set text(size: 48pt, fill: white)
  box(width: 100%, height: 100%)[
    #place(center, dy: 20%)[= Wave]
    #place(center + bottom)[
      #polygon(
        fill: blue.transparentize(90%),
        stroke: blue,
        (0%, 0%),
        ..wave_gen(calc.sin, 1, 10%, t, 20),
        (100%, 0%),
      )
    ]
    #place(center + bottom)[
      #polygon(
        fill: red.transparentize(90%),
        stroke: red,
        (0%, 0%),
        ..wave_gen(calc.cos, 1, 10%, t, 20),
        (100%, 0%),
      )
    ]
  ]
}
