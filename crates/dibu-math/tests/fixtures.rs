//! Renderiza los ejemplos de `dibu-comp` y `dibu-math` con ambas paletas y verifica que
//! los SVG sean sanos; además, que entradas inválidas den error y nunca un pánico.

use dibu_core::{leer_bloques, Registro};

fn registro() -> Registro {
    let mut r = Registro::new();
    dibu_comp::registrar(&mut r);
    dibu_math::registrar(&mut r);
    r
}

fn revisar(svg: &str, donde: &str) {
    assert!(
        svg.starts_with("<svg") && svg.ends_with("</svg>"),
        "{donde}: SVG mal formado"
    );
    // Sólo se revisan las etiquetas (atributos), no el texto visible.
    let mut etiquetas = String::new();
    let mut dentro = false;
    for ch in svg.chars() {
        match ch {
            '<' => dentro = true,
            '>' => dentro = false,
            _ if dentro => etiquetas.push(ch),
            _ => {}
        }
    }
    for malo in ["NaN", "inf"] {
        assert!(!etiquetas.contains(malo), "{donde}: atributo con {malo}");
    }
    // Etiquetas <text> balanceadas.
    assert_eq!(
        svg.matches("<text").count(),
        svg.matches("</text>").count(),
        "{donde}: <text> sin cerrar"
    );
}

#[test]
fn ejemplos_renderizan_con_ambas_paletas() {
    let reg = registro();
    let mut total = 0;
    for f in [
        "../../fixtures/diagramas_comp.yaml",
        "../../fixtures/diagramas_math.yaml",
    ] {
        let texto = std::fs::read_to_string(f).unwrap();
        for (i, b) in leer_bloques(&texto).unwrap().iter().enumerate() {
            for tinta in [true, false] {
                let svg = reg
                    .renderizar(b, tinta)
                    .unwrap_or_else(|e| panic!("{f} bloque {i}: {}", e.0));
                revisar(&svg, &format!("{f} bloque {i}"));
            }
            total += 1;
        }
    }
    assert!(
        total >= 45,
        "se esperaban al menos 45 ejemplos, hay {total}"
    );
    // Todos los tipos nuevos tienen al menos un ejemplo.
    let usados: String = [
        "../../fixtures/diagramas_comp.yaml",
        "../../fixtures/diagramas_math.yaml",
    ]
    .iter()
    .map(|f| std::fs::read_to_string(f).unwrap())
    .collect();
    for t in reg.tipos() {
        assert!(
            usados.contains(&format!("type: {t}\n")),
            "falta un ejemplo de {t}"
        );
    }
}

#[test]
fn ningun_tipo_nuevo_pisa_uno_existente() {
    let mut existentes = Registro::new();
    dibu_cs::registrar(&mut existentes);
    dibu_control::registrar(&mut existentes);
    dibu_chem::registrar(&mut existentes);
    for t in registro().tipos() {
        assert!(
            !existentes.contiene(t),
            "el tipo {t:?} ya existe en dibu-cs/control/chem"
        );
    }
}

#[test]
fn entradas_invalidas_dan_error_sin_panico() {
    let reg = registro();
    let casos = [
        "type: truth_table",
        "type: truth_table\nexpr: 'A + '",
        "type: truth_table\nexpr: 'A B C D E F G'",
        "type: karnaugh\nexpr: 'A'",
        "type: karnaugh\nvars: [A, B]\nminterms: [9]",
        "type: logic_circuit\nexpr: '(('",
        "type: pipeline\ninstructions: []",
        "type: ieee754\nvalue: abc",
        "type: ieee754\nbits: '0xFFFFFFFFFF'",
        "type: cache\ncacheSize: 100\nblockSize: 8\naccesses: [1]",
        "type: cache\ncacheSize: 64\nblockSize: 128\naccesses: [1]",
        "type: cpu_scheduling\nprocesses: [{name: A, burst: 0}]",
        "type: cpu_scheduling\nalgorithm: rr\nquantum: 0\nprocesses: [{burst: 3}]",
        "type: page_table\npageSize: 1000\ntable: [1]\naddresses: [5]",
        "type: page_replacement\nframes: 0\nreferences: [1]",
        "type: subnetting\nnetwork: 10.0.0.0/30\nsubnets: [{hosts: 100}]",
        "type: subnetting\nnetwork: 300.1.1.1/24\ncount: 2",
        "type: transaction_schedule\nschedule: [X1(A)]",
        "type: heap\nops: [extract]",
        "type: hash_table\nsize: 0\nkeys: [1]",
        "type: dp_table\nproblem: lcs\na: ''\nb: x",
        "type: dijkstra\nedges: ['A-B:-1']",
        "type: turing\ntransitions: ['q0,1 -> q1']",
        "type: signal\nexpr: 'sin(x)'",
        "type: signal\nexpr: '2 +'\n",
        "type: signal\ndomain: discrete\nexpr: n\nrange: [0, 100000]",
        "type: convolution\nx: []\nh: [1]",
        "type: sampling\nfrequency: 1\nfs: 100000\nperiods: 40",
        "type: distribution\ndist: normal\nsd: 0",
        "type: distribution\ndist: uniform\na: 2\nb: 1",
        "type: venn\nsets: [A]",
        "type: venn\nsets: [A, B]\nshade: 'A ∩ Z'",
        "type: circuit\nnetwork: {parallel: [R1]}",
        "type: phasor\nphasors: []",
    ];
    for caso in casos {
        let b = &leer_bloques(caso).unwrap()[0];
        let r = std::panic::catch_unwind(|| reg.renderizar(b, true));
        assert!(r.is_ok(), "pánico con:\n{caso}");
        assert!(r.unwrap().is_err(), "se esperaba error con:\n{caso}");
    }
}

#[test]
fn casos_borde_validos() {
    let reg = registro();
    let casos = [
        "type: truth_table\nexpr: '1'\nvars: [A]",
        "type: karnaugh\nvars: [A, B]\nminterms: []",
        "type: karnaugh\nvars: [A, B]\nminterms: [0, 1, 2, 3]",
        "type: logic_circuit\nexpr: A",
        "type: logic_circuit\nexpr: '!A'",
        "type: ieee754\nvalue: 0",
        "type: ieee754\nvalue: inf\nformat: half",
        "type: ieee754\nvalue: 1e-40\nformat: single",
        "type: ieee754\nvalue: nan\nformat: double",
        "type: cpu_scheduling\nprocesses: [{burst: 1, arrival: 5}]",
        "type: page_replacement\nframes: 1\nreferences: [1, 1, 2]\nalgorithm: clock",
        "type: subnetting\nnetwork: 192.168.0.0/24\ncount: 1",
        "type: transaction_schedule\nschedule: 'R1(A), C1'",
        "type: heap\nvalues: []",
        "type: avl\nkeys: [5, 5, 5]",
        "type: bplus_tree\nkeys: [1]",
        "type: dp_table\nproblem: coins\ncoins: [5]\namount: 3",
        "type: dijkstra\nvertices: [A, B]\nedges: ['A->A:1']",
        "type: signal\nexpr: '1/t'\nrange: [-1, 1]",
        "type: fourier\nsequence: [0]",
        "type: distribution\ndist: poisson\nlambda: 0.1\ninterval: [0, 0]",
        "type: venn\nsets: [A, B, C]\nshade: U",
        "type: circuit\nnetwork: ['C1 = 1uF']\nsource: 'V = 5'",
        "type: phasor\nphasors: [{magnitude: 0}]",
    ];
    for caso in casos {
        let b = &leer_bloques(caso).unwrap()[0];
        let svg = reg
            .renderizar(b, true)
            .unwrap_or_else(|e| panic!("{caso}\n→ {}", e.0));
        revisar(&svg, caso);
    }
}
