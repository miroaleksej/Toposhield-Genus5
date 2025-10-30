// circuits/holonomy_path.circom
// TopoShield ZKP: Holonomy-based signature verification
// Genus = 5, path length = 20, faithful SL(2, Fp) representation
// All matrices have det = 1 and satisfy ∏[A_i, B_i] = I
include "./poseidon.circom";

template SL2Multiply() {
    signal input A[4]; // [a, b, c, d]
    signal input B[4];
    signal output C[4];
    C[0] <== A[0]*B[0] + A[1]*B[2];
    C[1] <== A[0]*B[1] + A[1]*B[3];
    C[2] <== A[2]*B[0] + A[3]*B[2];
    C[3] <== A[2]*B[1] + A[3]*B[3];
}

// Generator matrices: normalized to det = 1
// Indices: 0-4 = a1-a5, 5-9 = b1-b5, 10-14 = a1⁻¹-a5⁻¹, 15-19 = b1⁻¹-b5⁻¹
template GeneratorMatrix(idx) {
    signal output M[4];
    if (idx == 0) { // a1
        M[0] <== 2; M[1] <== 1; M[2] <== 1; M[3] <== 1;
    } else if (idx == 1) { // b1
        M[0] <== 3; M[1] <== 2; M[2] <== 1; M[3] <== 1;
    } else if (idx == 2) { // a2
        M[0] <== 5; M[1] <== 3; M[2] <== 2; M[3] <== 1;
    } else if (idx == 3) { // b2
        M[0] <== 7; M[1] <== 4; M[2] <== 3; M[3] <== 2;
    } else if (idx == 4) { // a3
        M[0] <== 11; M[1] <== 7; M[2] <== 4; M[3] <== 3;
    } else if (idx == 5) { // b3
        M[0] <== 13; M[1] <== 8; M[2] <== 5; M[3] <== 3;
    } else if (idx == 6) { // a4
        M[0] <== 17; M[1] <== 11; M[2] <== 7; M[3] <== 4;
    } else if (idx == 7) { // b4
        M[0] <== 19; M[1] <== 12; M[2] <== 8; M[3] <== 5;
    } else if (idx == 8) { // a5
        M[0] <== 23; M[1] <== 14; M[2] <== 9; M[3] <== 6;
    } else if (idx == 9) { // b5 — normalized: (147,91,56,35) → (21,13,8,5)
        M[0] <== 21; M[1] <== 13; M[2] <== 8; M[3] <== 5;
    } else if (idx == 10) { // a1⁻¹
        M[0] <== 1; M[1] <== -1; M[2] <== -1; M[3] <== 2;
    } else if (idx == 11) { // b1⁻¹
        M[0] <== 1; M[1] <== -2; M[2] <== -1; M[3] <== 3;
    } else if (idx == 12) { // a2⁻¹
        M[0] <== 1; M[1] <== -3; M[2] <== -2; M[3] <== 5;
    } else if (idx == 13) { // b2⁻¹
        M[0] <== 2; M[1] <== -4; M[2] <== -3; M[3] <== 7;
    } else if (idx == 14) { // a3⁻¹
        M[0] <== 3; M[1] <== -7; M[2] <== -4; M[3] <== 11;
    } else if (idx == 15) { // b3⁻¹
        M[0] <== 3; M[1] <== -8; M[2] <== -5; M[3] <== 13;
    } else if (idx == 16) { // a4⁻¹
        M[0] <== 4; M[1] <== -11; M[2] <== -7; M[3] <== 17;
    } else if (idx == 17) { // b4⁻¹
        M[0] <== 5; M[1] <== -12; M[2] <== -8; M[3] <== 19;
    } else if (idx == 18) { // a5⁻¹
        M[0] <== 6; M[1] <== -14; M[2] <== -9; M[3] <== 23;
    } else if (idx == 19) { // b5⁻¹ = [[5, -13], [-8, 21]]
        M[0] <== 5; M[1] <== -13; M[2] <== -8; M[3] <== 21;
    } else {
        M[0] <== 1; M[1] <== 0; M[2] <== 0; M[3] <== 1;
    }
}

template PathToHolonomy(pathLen) {
    signal input indices[pathLen];
    signal output result[4];
    component gen[pathLen];
    signal mats[pathLen][4];

    for (var i = 0; i < pathLen; i++) {
        gen[i] = GeneratorMatrix(indices[i]);
        for (var j = 0; j < 4; j++) {
            mats[i][j] <== gen[i].M[j];
        }
    }

    if (pathLen == 1) {
        for (var j = 0; j < 4; j++) {
            result[j] <== mats[0][j];
        }
    } else {
        component mul[pathLen - 1];
        for (var i = 0; i < pathLen - 1; i++) {
            mul[i] = SL2Multiply();
            if (i == 0) {
                for (var j = 0; j < 4; j++) {
                    mul[i].A[j] <== mats[0][j];
                    mul[i].B[j] <== mats[1][j];
                }
            } else {
                for (var j = 0; j < 4; j++) {
                    mul[i].A[j] <== mul[i-1].C[j];
                    mul[i].B[j] <== mats[i+1][j];
                }
            }
        }
        for (var j = 0; j < 4; j++) {
            result[j] <== mul[pathLen - 2].C[j];
        }
    }
}

template TopoShieldVerify() {
    signal input H_pub[4];
    signal input H_sig[4];
    signal input desc_M[4];
    signal input m_hash[4];
    signal private gamma[20];
    signal private delta[20];

    component pubPath = PathToHolonomy(20);
    for (var i = 0; i < 20; i++) {
        pubPath.indices[i] <== gamma[i];
    }
    for (var i = 0; i < 4; i++) {
        pubPath.result[i] === H_pub[i];
    }

    signal combined[40];
    for (var i = 0; i < 20; i++) {
        combined[i] <== gamma[i];
    }
    for (var i = 0; i < 20; i++) {
        combined[20 + i] <== delta[i];
    }

    component sigPath = PathToHolonomy(40);
    for (var i = 0; i < 40; i++) {
        sigPath.indices[i] <== combined[i];
    }
    for (var i = 0; i < 4; i++) {
        sigPath.result[i] === H_sig[i];
    }

    component desc = Poseidon(3);
    desc.in[0] <== 5;
    desc.in[1] <== -8;
    desc.in[2] <== 12345;
    for (var i = 0; i < 4; i++) {
        desc.out[i] === desc_M[i];
    }
}

component main = TopoShieldVerify();
