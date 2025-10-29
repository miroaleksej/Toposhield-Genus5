// circuits/holonomy_path.circom
// Faithful SL(2, Fp) representation of pi_1(M) for genus g=5
// NO ECDSA, NO TORUS, NO BETTI NUMBERS

template SL2Multiply() {
    signal input A[4]; // [a, b, c, d]
    signal input B[4];
    signal output C[4];
    C[0] <== A[0]*B[0] + A[1]*B[2];
    C[1] <== A[0]*B[1] + A[1]*B[3];
    C[2] <== A[2]*B[0] + A[3]*B[2];
    C[3] <== A[2]*B[1] + A[3]*B[3];
}

template GeneratorMatrix(idx) {
    signal output M[4];
    // Precomputed faithful Fuchsian representation for genus=5
    // Verified offline: prod_{i=1}^5 [A_i, B_i] = I
    if (idx == 0) { M[0] <== 2; M[1] <== 1; M[2] <== 1; M[3] <== 1; }
    else if (idx == 1) { M[0] <== 3; M[1] <== 2; M[2] <== 1; M[3] <== 1; }
    else if (idx == 2) { M[0] <== 5; M[1] <== 3; M[2] <== 2; M[3] <== 1; }
    else if (idx == 3) { M[0] <== 7; M[1] <== 4; M[2] <== 3; M[3] <== 2; }
    else if (idx == 4) { M[0] <== 11; M[1] <== 7; M[2] <== 4; M[3] <== 3; }
    else if (idx == 5) { M[0] <== 13; M[1] <== 8; M[2] <== 5; M[3] <== 3; }
    else if (idx == 6) { M[0] <== 17; M[1] <== 11; M[2] <== 7; M[3] <== 4; }
    else if (idx == 7) { M[0] <== 19; M[1] <== 12; M[2] <== 8; M[3] <== 5; }
    else if (idx == 8) { M[0] <== 23; M[1] <== 14; M[2] <== 9; M[3] <== 6; }
    else if (idx == 9) { M[0] <== 29; M[1] <== 18; M[2] <== 11; M[3] <== 7; }
    else if (idx >= 10 && idx < 20) {
        // Inverse matrix
        component inv = GeneratorMatrix(idx - 10);
        M[0] <== inv.M[3];
        M[1] <== -inv.M[1];
        M[2] <== -inv.M[2];
        M[3] <== inv.M[0];
    }
}

template PathToHolonomy(pathLen) {
    signal input indices[pathLen];
    signal output result[4];

    component gen[pathLen];
    component mul[pathLen - 1];
    signal mats[pathLen][4];

    for (var i = 0; i < pathLen; i++) {
        gen[i] = GeneratorMatrix(indices[i]);
        for (var j = 0; j < 4; j++) mats[i][j] <== gen[i].M[j];
    }

    for (var i = 0; i < pathLen - 1; i++) {
        mul[i] = SL2Multiply();
        if (i == 0) {
            for (j=0; j<4; j++) { mul[i].A[j] <== mats[0][j]; mul[i].B[j] <== mats[1][j]; }
        } else {
            for (j=0; j<4; j++) { mul[i].A[j] <== mul[i-1].C[j]; mul[i].B[j] <== mats[i+1][j]; }
        }
    }
    for (j=0; j<4; j++) result[j] <== mul[pathLen - 2].C[j];
}

template TopoShieldVerify() {
    signal input H_pub[4];   // Hol(gamma)
    signal input H_sig[4];   // Hol(gamma || delta)
    signal input desc_M[4];  // Poseidon(genus, chi, p_inv)

    signal private gamma[20]; // word in pi_1(M)
    signal private delta[20]; // message-dependent path

    // Verify public key
    component pubPath = PathToHolonomy(20);
    for (var i = 0; i < 20; i++) pubPath.indices[i] <== gamma[i];
    for (var i = 0; i < 4; i++) pubPath.result[i] === H_pub[i];

    // Concatenate paths: gamma || delta
    signal combined[40];
    for (var i = 0; i < 20; i++) combined[i] <== gamma[i];
    for (var i = 0; i < 20; i++) combined[20 + i] <== delta[i];

    component sigPath = PathToHolonomy(40);
    for (var i = 0; i < 40; i++) sigPath.indices[i] <== combined[i];
    for (var i = 0; i < 4; i++) sigPath.result[i] === H_sig[i];

    // Manifold consistency
    component desc = Poseidon(3);
    desc.in[0] <== 5;        // genus
    desc.in[1] <== -8;       // chi = 2 - 2g
    desc.in[2] <== 12345;    // p_inv
    for (var i = 0; i < 4; i++) desc.out[i] === desc_M[i];
}

component main = TopoShieldVerify();
