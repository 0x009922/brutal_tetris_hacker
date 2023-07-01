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

interface Tetra {
    items: [number, number][]
    col_shift: number
}

const tetras: Tetra[] = []
let last: { tetra: Tetra, y: number } | undefined

const lines = raw.split('\n')
for (const line of lines) {
    const trimmed = line.trimEnd();
    if (trimmed) {
        if (!last) {
            last = { tetra: { items: [], col_shift: 0 }, y: -1 }
        }
        last.y ++;

        for (let i = 0; i < trimmed.length; i++) {
            if (trimmed[i] !== ' ') {
                const { tetra } = last
                if (tetra.items.length === 0) {
                    tetra.col_shift = i;
                }
                tetra.items.push([last.y, i])
            }
        }

        if (last.tetra.items.length > 4) throw new Error(`Too many points in tetra`)
    } else {
        if (last) {
            if (last.tetra.items.length < 4) throw new Error(`Bad tetra: ${last.tetra.items.map(x => `(${x[0]}, ${x[1]})`).join(', ')}`)
            tetras.push(last.tetra)
            last = undefined
        }
    }
}

last && tetras.push(last.tetra)

console.log(tetras.map(x => `tetra!(${x.items.map(([row, col]) => `(${row}, ${col})`).join(', ')}, ${x.col_shift})`).join(',\n'))