const raw = `
xx
xx

xxxx

x
x
x
x


xxx
 x

x
xx
x

 x
xxx

 x
xx
 x


xxx
x

x
x
xx

  x
xxx

xx
 x
 x

xxx
  x

 x
 x
xx

x
xxx

xx
x
x


 xx
xx

x
xx
 x


xx
 xx
 
 x
xx
x
`

interface Tetro {
    items: [number, number][]
}

const tetros: Tetro[] = []
let last: { tetro: Tetro, y: number } | undefined

const lines = raw.split('\n')
for (const line of lines) {
    const trimmed = line.trimEnd();
    if (trimmed) {
        if (!last) {
            last = { tetro: { items: []}, y: -1 }
        }
        last.y ++;

        for (let i = 0; i < trimmed.length; i++) {
            if (trimmed[i] !== ' ') {
                last.tetro.items.push([last.y, i])
            }
        }

        if (last.tetro.items.length > 4) throw new Error(`Too many points in tetro`)
    } else {
        if (last) {
            if (last.tetro.items.length < 4) throw new Error(`Bad tetro: ${last.tetro.items.map(x => `(${x[0]}, ${x[1]})`).join(', ')}`)
            tetros.push(last.tetro)
            last = undefined
        }
    }
}

last && tetros.push(last.tetro)

console.log(tetros.map(x => `tetro!(${x.items.map(([row, col]) => `(${row}, ${col})`).join(', ')})`).join(',\n'))