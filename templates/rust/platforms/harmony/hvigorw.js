"use strict";

var e, t = require("fs"), r = require("path"), n = require("process"), o = require("crypto"), i = require("child_process"), a = require("os"), s = require("constants"), u = require("stream"), l = require("util"), c = require("assert"), f = require("tty"), d = require("url"), p = require("zlib"), h = require("net"), v = require("fs/promises"), g = "undefined" != typeof globalThis ? globalThis : "undefined" != typeof window ? window : "undefined" != typeof global ? global : "undefined" != typeof self ? self : {}, m = {}, y = {}, _ = {};

e = _, Object.defineProperty(e, "__esModule", {
    value: !0
}), e.isCI = void 0, e.isCI = function() {
    return !("false" === process.env.CI || !(process.env.BUILD_ID || process.env.BUILD_NUMBER || process.env.CI || process.env.CI_APP_ID || process.env.CI_BUILD_ID || process.env.CI_BUILD_NUMBER || process.env.CI_NAME || process.env.CONTINUOUS_INTEGRATION || process.env.RUN_ID || e.name));
};

var E = {};

!function(e) {
    var t = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.isSubPath = void 0;
    const n = t(r);
    e.isSubPath = function(e, t) {
        try {
            const r = n.default.relative(e, t);
            if ("" === r) {
                return !0;
            }
            const o = r.split(n.default.sep);
            for (let e of o) {
                if (".." !== e) {
                    return !1;
                }
            }
            return !0;
        } catch (e) {
            return !1;
        }
    };
}(E);

var b = {};

!function(e) {
    var r = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.hashObject = e.getSafeJsonStringIfy = e.hashFile = e.hash = e.createHash = void 0;
    const n = r(o), i = r(t);
    e.createHash = (e = "sha256") => n.default.createHash(e);
    e.hash = (t, r) => (0, e.createHash)(r).update(t).digest("hex");
    e.hashFile = (t, r) => {
        if (i.default.existsSync(t)) {
            return (0, e.hash)(i.default.readFileSync(t, "utf-8"), r);
        }
    };
    e.getSafeJsonStringIfy = e => {
        const t = new WeakSet;
        function r(e) {
            if (null === e || "object" != typeof e) {
                return "bigint" == typeof e ? null : e;
            }
            if (t.has(e)) {
                return null;
            }
            if (t.add(e), Array.isArray(e)) {
                return e.map(r);
            }
            const n = {};
            return Object.keys(e).sort().forEach(t => {
                "function" != typeof e[t] && "symbol" != typeof e[t] && (n[t] = r(e[t]));
            }), n;
        }
        return JSON.stringify(e, function(e, t) {
            return r(t);
        });
    };
    e.hashObject = t => n.default.createHash("sha256").update((0, e.getSafeJsonStringIfy)(t)).digest("hex");
}(b);

var w = {};

Object.defineProperty(w, "__esModule", {
    value: !0
});

w.default = {
    preset: "ts-jest",
    testEnvironment: "node",
    maxConcurrency: 8,
    maxWorkers: 8,
    testPathIgnorePatterns: [ "/node_modules/", "/test/resources/", "/test/temp/" ],
    testTimeout: 3e5,
    testMatch: [ "**/e2e-test/**/*.ts?(x)", "**/jest-test/**/*.ts?(x)", "**/__tests__/**/*.ts?(x)", "**/?(*.)?(long|unit)+(spec|test).ts?(x)" ],
    collectCoverageFrom: [ "**/src/**/*.js" ],
    coverageReporters: [ "json", "lcov", "text", "clover" ]
};

var D = {}, S = {};

!function(e) {
    var t = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.maxPathLength = e.isMac = e.isLinux = e.isWindows = void 0;
    const r = t(a);
    function n() {
        return "Windows_NT" === r.default.type();
    }
    function o() {
        return "Darwin" === r.default.type();
    }
    e.isWindows = n, e.isLinux = function() {
        return "Linux" === r.default.type();
    }, e.isMac = o, e.maxPathLength = function() {
        return o() ? 1016 : n() ? 259 : 4095;
    };
}(S), function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.getOsLanguage = e.countryEnum = void 0;
    const t = i, r = S;
    var n;
    let o;
    !function(e) {
        e.CN = "cn", e.EN = "en";
    }(n = e.countryEnum || (e.countryEnum = {})), e.getOsLanguage = function() {
        if (o) {
            return o;
        }
        let e = n.CN;
        return (0, r.isWindows)() ? e = function() {
            const e = (0, t.spawnSync)("wmic", [ "os", "get", "locale" ]);
            return 0 !== e.status || 2052 === Number.parseInt(e.stdout.toString().replace("Locale", ""), 16) ? n.CN : n.EN;
        }() : (0, r.isMac)() ? e = function() {
            const e = (0, t.spawnSync)("defaults", [ "read", "-globalDomain", "AppleLocale" ]);
            return 0 !== e.status || e.stdout.toString().indexOf("zh_CN") >= 0 ? n.CN : n.EN;
        }() : (0, r.isLinux)() && (e = function() {
            var e;
            const r = (0, t.spawnSync)("locale");
            if (0 !== r.status) {
                return n.CN;
            }
            const o = {};
            for (const t of r.stdout.toString().split("\n")) {
                const [r, n] = t.split("=");
                o[r] = null !== (e = null == n ? void 0 : n.replace(/^"|"$/g, "")) && void 0 !== e ? e : "";
            }
            const i = o.LC_ALL || o.LC_MESSAGES || o.LANG || o.LANGUAGE;
            return i && i.indexOf("zh_CN") >= 0 ? n.CN : n.EN;
        }()), process.env["user.country"] && (e = "CN" === process.env["user.country"].toString() ? n.CN : n.EN), 
        o = e, e;
    };
}(D);

var A = {}, O = {}, C = "undefined" != typeof document ? document.currentScript : null;

const x = 256, F = 256, M = -2, P = -5;

function I(e) {
    return k(e.map(([e, t]) => new Array(e).fill(t, 0, e)));
}

function k(e) {
    return e.reduce((e, t) => e.concat(Array.isArray(t) ? k(t) : t), []);
}

const R = [ 0, 1, 2, 3 ].concat(...I([ [ 2, 4 ], [ 2, 5 ], [ 4, 6 ], [ 4, 7 ], [ 8, 8 ], [ 8, 9 ], [ 16, 10 ], [ 16, 11 ], [ 32, 12 ], [ 32, 13 ], [ 64, 14 ], [ 64, 15 ], [ 2, 0 ], [ 1, 16 ], [ 1, 17 ], [ 2, 18 ], [ 2, 19 ], [ 4, 20 ], [ 4, 21 ], [ 8, 22 ], [ 8, 23 ], [ 16, 24 ], [ 16, 25 ], [ 32, 26 ], [ 32, 27 ], [ 64, 28 ], [ 64, 29 ] ]));

function T() {
    const e = this;
    function t(e, t) {
        let r = 0;
        do {
            r |= 1 & e, e >>>= 1, r <<= 1;
        } while (--t > 0);
        return r >>> 1;
    }
    e.build_tree = function(r) {
        const n = e.dyn_tree, o = e.stat_desc.static_tree, i = e.stat_desc.elems;
        let a, s, u, l = -1;
        for (r.heap_len = 0, r.heap_max = 573, a = 0; a < i; a++) {
            0 !== n[2 * a] ? (r.heap[++r.heap_len] = l = a, r.depth[a] = 0) : n[2 * a + 1] = 0;
        }
        for (;r.heap_len < 2; ) {
            u = r.heap[++r.heap_len] = l < 2 ? ++l : 0, n[2 * u] = 1, r.depth[u] = 0, r.opt_len--, 
            o && (r.static_len -= o[2 * u + 1]);
        }
        for (e.max_code = l, a = Math.floor(r.heap_len / 2); a >= 1; a--) {
            r.pqdownheap(n, a);
        }
        u = i;
        do {
            a = r.heap[1], r.heap[1] = r.heap[r.heap_len--], r.pqdownheap(n, 1), s = r.heap[1], 
            r.heap[--r.heap_max] = a, r.heap[--r.heap_max] = s, n[2 * u] = n[2 * a] + n[2 * s], 
            r.depth[u] = Math.max(r.depth[a], r.depth[s]) + 1, n[2 * a + 1] = n[2 * s + 1] = u, 
            r.heap[1] = u++, r.pqdownheap(n, 1);
        } while (r.heap_len >= 2);
        r.heap[--r.heap_max] = r.heap[1], function(t) {
            const r = e.dyn_tree, n = e.stat_desc.static_tree, o = e.stat_desc.extra_bits, i = e.stat_desc.extra_base, a = e.stat_desc.max_length;
            let s, u, l, c, f, d, p = 0;
            for (c = 0; c <= 15; c++) {
                t.bl_count[c] = 0;
            }
            for (r[2 * t.heap[t.heap_max] + 1] = 0, s = t.heap_max + 1; s < 573; s++) {
                u = t.heap[s], c = r[2 * r[2 * u + 1] + 1] + 1, c > a && (c = a, p++), r[2 * u + 1] = c, 
                u > e.max_code || (t.bl_count[c]++, f = 0, u >= i && (f = o[u - i]), d = r[2 * u], 
                t.opt_len += d * (c + f), n && (t.static_len += d * (n[2 * u + 1] + f)));
            }
            if (0 !== p) {
                do {
                    for (c = a - 1; 0 === t.bl_count[c]; ) {
                        c--;
                    }
                    t.bl_count[c]--, t.bl_count[c + 1] += 2, t.bl_count[a]--, p -= 2;
                } while (p > 0);
                for (c = a; 0 !== c; c--) {
                    for (u = t.bl_count[c]; 0 !== u; ) {
                        l = t.heap[--s], l > e.max_code || (r[2 * l + 1] != c && (t.opt_len += (c - r[2 * l + 1]) * r[2 * l], 
                        r[2 * l + 1] = c), u--);
                    }
                }
            }
        }(r), function(e, r, n) {
            const o = [];
            let i, a, s, u = 0;
            for (i = 1; i <= 15; i++) {
                o[i] = u = u + n[i - 1] << 1;
            }
            for (a = 0; a <= r; a++) {
                s = e[2 * a + 1], 0 !== s && (e[2 * a] = t(o[s]++, s));
            }
        }(n, e.max_code, r.bl_count);
    };
}

function j(e, t, r, n, o) {
    const i = this;
    i.static_tree = e, i.extra_bits = t, i.extra_base = r, i.elems = n, i.max_length = o;
}

T._length_code = [ 0, 1, 2, 3, 4, 5, 6, 7 ].concat(...I([ [ 2, 8 ], [ 2, 9 ], [ 2, 10 ], [ 2, 11 ], [ 4, 12 ], [ 4, 13 ], [ 4, 14 ], [ 4, 15 ], [ 8, 16 ], [ 8, 17 ], [ 8, 18 ], [ 8, 19 ], [ 16, 20 ], [ 16, 21 ], [ 16, 22 ], [ 16, 23 ], [ 32, 24 ], [ 32, 25 ], [ 32, 26 ], [ 31, 27 ], [ 1, 28 ] ])), 
T.base_length = [ 0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 12, 14, 16, 20, 24, 28, 32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 0 ], 
T.base_dist = [ 0, 1, 2, 3, 4, 6, 8, 12, 16, 24, 32, 48, 64, 96, 128, 192, 256, 384, 512, 768, 1024, 1536, 2048, 3072, 4096, 6144, 8192, 12288, 16384, 24576 ], 
T.d_code = function(e) {
    return e < 256 ? R[e] : R[256 + (e >>> 7)];
}, T.extra_lbits = [ 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0 ], 
T.extra_dbits = [ 0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13 ], 
T.extra_blbits = [ 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 3, 7 ], T.bl_order = [ 16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15 ];

const L = I([ [ 144, 8 ], [ 112, 9 ], [ 24, 7 ], [ 8, 8 ] ]);

j.static_ltree = k([ 12, 140, 76, 204, 44, 172, 108, 236, 28, 156, 92, 220, 60, 188, 124, 252, 2, 130, 66, 194, 34, 162, 98, 226, 18, 146, 82, 210, 50, 178, 114, 242, 10, 138, 74, 202, 42, 170, 106, 234, 26, 154, 90, 218, 58, 186, 122, 250, 6, 134, 70, 198, 38, 166, 102, 230, 22, 150, 86, 214, 54, 182, 118, 246, 14, 142, 78, 206, 46, 174, 110, 238, 30, 158, 94, 222, 62, 190, 126, 254, 1, 129, 65, 193, 33, 161, 97, 225, 17, 145, 81, 209, 49, 177, 113, 241, 9, 137, 73, 201, 41, 169, 105, 233, 25, 153, 89, 217, 57, 185, 121, 249, 5, 133, 69, 197, 37, 165, 101, 229, 21, 149, 85, 213, 53, 181, 117, 245, 13, 141, 77, 205, 45, 173, 109, 237, 29, 157, 93, 221, 61, 189, 125, 253, 19, 275, 147, 403, 83, 339, 211, 467, 51, 307, 179, 435, 115, 371, 243, 499, 11, 267, 139, 395, 75, 331, 203, 459, 43, 299, 171, 427, 107, 363, 235, 491, 27, 283, 155, 411, 91, 347, 219, 475, 59, 315, 187, 443, 123, 379, 251, 507, 7, 263, 135, 391, 71, 327, 199, 455, 39, 295, 167, 423, 103, 359, 231, 487, 23, 279, 151, 407, 87, 343, 215, 471, 55, 311, 183, 439, 119, 375, 247, 503, 15, 271, 143, 399, 79, 335, 207, 463, 47, 303, 175, 431, 111, 367, 239, 495, 31, 287, 159, 415, 95, 351, 223, 479, 63, 319, 191, 447, 127, 383, 255, 511, 0, 64, 32, 96, 16, 80, 48, 112, 8, 72, 40, 104, 24, 88, 56, 120, 4, 68, 36, 100, 20, 84, 52, 116, 3, 131, 67, 195, 35, 163, 99, 227 ].map((e, t) => [ e, L[t] ]));

const N = I([ [ 30, 5 ] ]);

j.static_dtree = k([ 0, 16, 8, 24, 4, 20, 12, 28, 2, 18, 10, 26, 6, 22, 14, 30, 1, 17, 9, 25, 5, 21, 13, 29, 3, 19, 11, 27, 7, 23 ].map((e, t) => [ e, N[t] ])), 
j.static_l_desc = new j(j.static_ltree, T.extra_lbits, 257, 286, 15), j.static_d_desc = new j(j.static_dtree, T.extra_dbits, 0, 30, 15), 
j.static_bl_desc = new j(null, T.extra_blbits, 0, 19, 7);

function B(e, t, r, n, o) {
    const i = this;
    i.good_length = e, i.max_lazy = t, i.nice_length = r, i.max_chain = n, i.func = o;
}

const U = [ new B(0, 0, 0, 0, 0), new B(4, 4, 8, 4, 1), new B(4, 5, 16, 8, 1), new B(4, 6, 32, 32, 1), new B(4, 4, 16, 16, 2), new B(8, 16, 32, 32, 2), new B(8, 16, 128, 128, 2), new B(8, 32, 128, 256, 2), new B(32, 128, 258, 1024, 2), new B(32, 258, 258, 4096, 2) ], z = [ "need dictionary", "stream end", "", "", "stream error", "data error", "", "buffer error", "", "" ], H = 113, $ = 666, G = 258, W = 262;

function V(e, t, r, n) {
    const o = e[2 * t], i = e[2 * r];
    return o < i || o == i && n[t] <= n[r];
}

function K() {
    const e = this;
    let t, r, n, o, i, a, s, u, l, c, f, d, p, h, v, g, m, y, _, E, b, w, D, S, A, O, C, I, k, R, L, N, B;
    const K = new T, q = new T, J = new T;
    let X, Z, Y, Q, ee, te;
    function re() {
        let t;
        for (t = 0; t < 286; t++) {
            L[2 * t] = 0;
        }
        for (t = 0; t < 30; t++) {
            N[2 * t] = 0;
        }
        for (t = 0; t < 19; t++) {
            B[2 * t] = 0;
        }
        L[512] = 1, e.opt_len = e.static_len = 0, Z = Y = 0;
    }
    function ne(e, t) {
        let r, n = -1, o = e[1], i = 0, a = 7, s = 4;
        0 === o && (a = 138, s = 3), e[2 * (t + 1) + 1] = 65535;
        for (let u = 0; u <= t; u++) {
            r = o, o = e[2 * (u + 1) + 1], ++i < a && r == o || (i < s ? B[2 * r] += i : 0 !== r ? (r != n && B[2 * r]++, 
            B[32]++) : i <= 10 ? B[34]++ : B[36]++, i = 0, n = r, 0 === o ? (a = 138, s = 3) : r == o ? (a = 6, 
            s = 3) : (a = 7, s = 4));
        }
    }
    function oe(t) {
        e.pending_buf[e.pending++] = t;
    }
    function ie(e) {
        oe(255 & e), oe(e >>> 8 & 255);
    }
    function ae(e, t) {
        let r;
        const n = t;
        te > 16 - n ? (r = e, ee |= r << te & 65535, ie(ee), ee = r >>> 16 - te, te += n - 16) : (ee |= e << te & 65535, 
        te += n);
    }
    function se(e, t) {
        const r = 2 * e;
        ae(65535 & t[r], 65535 & t[r + 1]);
    }
    function ue(e, t) {
        let r, n, o = -1, i = e[1], a = 0, s = 7, u = 4;
        for (0 === i && (s = 138, u = 3), r = 0; r <= t; r++) {
            if (n = i, i = e[2 * (r + 1) + 1], !(++a < s && n == i)) {
                if (a < u) {
                    do {
                        se(n, B);
                    } while (0 !== --a);
                } else {
                    0 !== n ? (n != o && (se(n, B), a--), se(16, B), ae(a - 3, 2)) : a <= 10 ? (se(17, B), 
                    ae(a - 3, 3)) : (se(18, B), ae(a - 11, 7));
                }
                a = 0, o = n, 0 === i ? (s = 138, u = 3) : n == i ? (s = 6, u = 3) : (s = 7, u = 4);
            }
        }
    }
    function le() {
        16 == te ? (ie(ee), ee = 0, te = 0) : te >= 8 && (oe(255 & ee), ee >>>= 8, te -= 8);
    }
    function ce(t, r) {
        let n, o, i;
        if (e.dist_buf[Z] = t, e.lc_buf[Z] = 255 & r, Z++, 0 === t ? L[2 * r]++ : (Y++, 
        t--, L[2 * (T._length_code[r] + x + 1)]++, N[2 * T.d_code(t)]++), !(8191 & Z) && C > 2) {
            for (n = 8 * Z, o = b - m, i = 0; i < 30; i++) {
                n += N[2 * i] * (5 + T.extra_dbits[i]);
            }
            if (n >>>= 3, Y < Math.floor(Z / 2) && n < Math.floor(o / 2)) {
                return !0;
            }
        }
        return Z == X - 1;
    }
    function fe(t, r) {
        let n, o, i, a, s = 0;
        if (0 !== Z) {
            do {
                n = e.dist_buf[s], o = e.lc_buf[s], s++, 0 === n ? se(o, t) : (i = T._length_code[o], 
                se(i + x + 1, t), a = T.extra_lbits[i], 0 !== a && (o -= T.base_length[i], ae(o, a)), 
                n--, i = T.d_code(n), se(i, r), a = T.extra_dbits[i], 0 !== a && (n -= T.base_dist[i], 
                ae(n, a)));
            } while (s < Z);
        }
        se(F, t), Q = t[513];
    }
    function de() {
        te > 8 ? ie(ee) : te > 0 && oe(255 & ee), ee = 0, te = 0;
    }
    function pe(t, r, n) {
        ae(0 + (n ? 1 : 0), 3), function(t, r) {
            de(), Q = 8, ie(r), ie(~r), e.pending_buf.set(u.subarray(t, t + r), e.pending), 
            e.pending += r;
        }(t, r);
    }
    function he(t, r, n) {
        let o, i, a = 0;
        C > 0 ? (K.build_tree(e), q.build_tree(e), a = function() {
            let t;
            for (ne(L, K.max_code), ne(N, q.max_code), J.build_tree(e), t = 18; t >= 3 && 0 === B[2 * T.bl_order[t] + 1]; t--) {}
            return e.opt_len += 3 * (t + 1) + 5 + 5 + 4, t;
        }(), o = e.opt_len + 3 + 7 >>> 3, i = e.static_len + 3 + 7 >>> 3, i <= o && (o = i)) : o = i = r + 5, 
        r + 4 <= o && -1 != t ? pe(t, r, n) : i == o ? (ae(2 + (n ? 1 : 0), 3), fe(j.static_ltree, j.static_dtree)) : (ae(4 + (n ? 1 : 0), 3), 
        function(e, t, r) {
            let n;
            for (ae(e - 257, 5), ae(t - 1, 5), ae(r - 4, 4), n = 0; n < r; n++) {
                ae(B[2 * T.bl_order[n] + 1], 3);
            }
            ue(L, e - 1), ue(N, t - 1);
        }(K.max_code + 1, q.max_code + 1, a + 1), fe(L, N)), re(), n && de();
    }
    function ve(e) {
        he(m >= 0 ? m : -1, b - m, e), m = b, t.flush_pending();
    }
    function ge() {
        let e, r, n, o;
        do {
            if (o = l - D - b, 0 === o && 0 === b && 0 === D) {
                o = i;
            } else if (-1 == o) {
                o--;
            } else if (b >= i + i - W) {
                u.set(u.subarray(i, i + i), 0), w -= i, b -= i, m -= i, e = p, n = e;
                do {
                    r = 65535 & f[--n], f[n] = r >= i ? r - i : 0;
                } while (0 !== --e);
                e = i, n = e;
                do {
                    r = 65535 & c[--n], c[n] = r >= i ? r - i : 0;
                } while (0 !== --e);
                o += i;
            }
            if (0 === t.avail_in) {
                return;
            }
            e = t.read_buf(u, b + D, o), D += e, D >= 3 && (d = 255 & u[b], d = (d << g ^ 255 & u[b + 1]) & v);
        } while (D < W && 0 !== t.avail_in);
    }
    function me(e) {
        let t, r, n = A, o = b, a = S;
        const l = b > i - W ? b - (i - W) : 0;
        let f = R;
        const d = s, p = b + G;
        let h = u[o + a - 1], v = u[o + a];
        S >= k && (n >>= 2), f > D && (f = D);
        do {
            if (t = e, u[t + a] == v && u[t + a - 1] == h && u[t] == u[o] && u[++t] == u[o + 1]) {
                o += 2, t++;
                do {} while (u[++o] == u[++t] && u[++o] == u[++t] && u[++o] == u[++t] && u[++o] == u[++t] && u[++o] == u[++t] && u[++o] == u[++t] && u[++o] == u[++t] && u[++o] == u[++t] && o < p);
                if (r = G - (p - o), o = p - G, r > a) {
                    if (w = e, a = r, r >= f) {
                        break;
                    }
                    h = u[o + a - 1], v = u[o + a];
                }
            }
        } while ((e = 65535 & c[e & d]) > l && 0 !== --n);
        return a <= D ? a : D;
    }
    function ye(t) {
        return t.total_in = t.total_out = 0, t.msg = null, e.pending = 0, e.pending_out = 0, 
        r = H, o = 0, K.dyn_tree = L, K.stat_desc = j.static_l_desc, q.dyn_tree = N, q.stat_desc = j.static_d_desc, 
        J.dyn_tree = B, J.stat_desc = j.static_bl_desc, ee = 0, te = 0, Q = 8, re(), function() {
            l = 2 * i, f[p - 1] = 0;
            for (let e = 0; e < p - 1; e++) {
                f[e] = 0;
            }
            O = U[C].max_lazy, k = U[C].good_length, R = U[C].nice_length, A = U[C].max_chain, 
            b = 0, m = 0, D = 0, y = S = 2, E = 0, d = 0;
        }(), 0;
    }
    e.depth = [], e.bl_count = [], e.heap = [], L = [], N = [], B = [], e.pqdownheap = function(t, r) {
        const n = e.heap, o = n[r];
        let i = r << 1;
        for (;i <= e.heap_len && (i < e.heap_len && V(t, n[i + 1], n[i], e.depth) && i++, 
        !V(t, o, n[i], e.depth)); ) {
            n[r] = n[i], r = i, i <<= 1;
        }
        n[r] = o;
    }, e.deflateInit = function(t, r, o, l, d, m) {
        return l || (l = 8), d || (d = 8), m || (m = 0), t.msg = null, -1 == r && (r = 6), 
        d < 1 || d > 9 || 8 != l || o < 9 || o > 15 || r < 0 || r > 9 || m < 0 || m > 2 ? M : (t.dstate = e, 
        a = o, i = 1 << a, s = i - 1, h = d + 7, p = 1 << h, v = p - 1, g = Math.floor((h + 3 - 1) / 3), 
        u = new Uint8Array(2 * i), c = [], f = [], X = 1 << d + 6, e.pending_buf = new Uint8Array(4 * X), 
        n = 4 * X, e.dist_buf = new Uint16Array(X), e.lc_buf = new Uint8Array(X), C = r, 
        I = m, ye(t));
    }, e.deflateEnd = function() {
        return 42 != r && r != H && r != $ ? M : (e.lc_buf = null, e.dist_buf = null, e.pending_buf = null, 
        f = null, c = null, u = null, e.dstate = null, r == H ? -3 : 0);
    }, e.deflateParams = function(e, t, r) {
        let n = 0;
        return -1 == t && (t = 6), t < 0 || t > 9 || r < 0 || r > 2 ? M : (U[C].func != U[t].func && 0 !== e.total_in && (n = e.deflate(1)), 
        C != t && (C = t, O = U[C].max_lazy, k = U[C].good_length, R = U[C].nice_length, 
        A = U[C].max_chain), I = r, n);
    }, e.deflateSetDictionary = function(e, t, n) {
        let o, a = n, l = 0;
        if (!t || 42 != r) {
            return M;
        }
        if (a < 3) {
            return 0;
        }
        for (a > i - W && (a = i - W, l = n - a), u.set(t.subarray(l, l + a), 0), b = a, 
        m = a, d = 255 & u[0], d = (d << g ^ 255 & u[1]) & v, o = 0; o <= a - 3; o++) {
            d = (d << g ^ 255 & u[o + 2]) & v, c[o & s] = f[d], f[d] = o;
        }
        return 0;
    }, e.deflate = function(l, h) {
        let A, x, k, R, T;
        if (h > 4 || h < 0) {
            return M;
        }
        if (!l.next_out || !l.next_in && 0 !== l.avail_in || r == $ && 4 != h) {
            return l.msg = z[4], M;
        }
        if (0 === l.avail_out) {
            return l.msg = z[7], P;
        }
        var L;
        if (t = l, R = o, o = h, 42 == r && (x = 8 + (a - 8 << 4) << 8, k = (C - 1 & 255) >> 1, 
        k > 3 && (k = 3), x |= k << 6, 0 !== b && (x |= 32), x += 31 - x % 31, r = H, oe((L = x) >> 8 & 255), 
        oe(255 & L)), 0 !== e.pending) {
            if (t.flush_pending(), 0 === t.avail_out) {
                return o = -1, 0;
            }
        } else if (0 === t.avail_in && h <= R && 4 != h) {
            return t.msg = z[7], P;
        }
        if (r == $ && 0 !== t.avail_in) {
            return l.msg = z[7], P;
        }
        if (0 !== t.avail_in || 0 !== D || 0 != h && r != $) {
            switch (T = -1, U[C].func) {
              case 0:
                T = function(e) {
                    let r, o = 65535;
                    for (o > n - 5 && (o = n - 5); ;) {
                        if (D <= 1) {
                            if (ge(), 0 === D && 0 == e) {
                                return 0;
                            }
                            if (0 === D) {
                                break;
                            }
                        }
                        if (b += D, D = 0, r = m + o, (0 === b || b >= r) && (D = b - r, b = r, ve(!1), 
                        0 === t.avail_out)) {
                            return 0;
                        }
                        if (b - m >= i - W && (ve(!1), 0 === t.avail_out)) {
                            return 0;
                        }
                    }
                    return ve(4 == e), 0 === t.avail_out ? 4 == e ? 2 : 0 : 4 == e ? 3 : 1;
                }(h);
                break;

              case 1:
                T = function(e) {
                    let r, n = 0;
                    for (;;) {
                        if (D < W) {
                            if (ge(), D < W && 0 == e) {
                                return 0;
                            }
                            if (0 === D) {
                                break;
                            }
                        }
                        if (D >= 3 && (d = (d << g ^ 255 & u[b + 2]) & v, n = 65535 & f[d], c[b & s] = f[d], 
                        f[d] = b), 0 !== n && (b - n & 65535) <= i - W && 2 != I && (y = me(n)), y >= 3) {
                            if (r = ce(b - w, y - 3), D -= y, y <= O && D >= 3) {
                                y--;
                                do {
                                    b++, d = (d << g ^ 255 & u[b + 2]) & v, n = 65535 & f[d], c[b & s] = f[d], f[d] = b;
                                } while (0 !== --y);
                                b++;
                            } else {
                                b += y, y = 0, d = 255 & u[b], d = (d << g ^ 255 & u[b + 1]) & v;
                            }
                        } else {
                            r = ce(0, 255 & u[b]), D--, b++;
                        }
                        if (r && (ve(!1), 0 === t.avail_out)) {
                            return 0;
                        }
                    }
                    return ve(4 == e), 0 === t.avail_out ? 4 == e ? 2 : 0 : 4 == e ? 3 : 1;
                }(h);
                break;

              case 2:
                T = function(e) {
                    let r, n, o = 0;
                    for (;;) {
                        if (D < W) {
                            if (ge(), D < W && 0 == e) {
                                return 0;
                            }
                            if (0 === D) {
                                break;
                            }
                        }
                        if (D >= 3 && (d = (d << g ^ 255 & u[b + 2]) & v, o = 65535 & f[d], c[b & s] = f[d], 
                        f[d] = b), S = y, _ = w, y = 2, 0 !== o && S < O && (b - o & 65535) <= i - W && (2 != I && (y = me(o)), 
                        y <= 5 && (1 == I || 3 == y && b - w > 4096) && (y = 2)), S >= 3 && y <= S) {
                            n = b + D - 3, r = ce(b - 1 - _, S - 3), D -= S - 1, S -= 2;
                            do {
                                ++b <= n && (d = (d << g ^ 255 & u[b + 2]) & v, o = 65535 & f[d], c[b & s] = f[d], 
                                f[d] = b);
                            } while (0 !== --S);
                            if (E = 0, y = 2, b++, r && (ve(!1), 0 === t.avail_out)) {
                                return 0;
                            }
                        } else if (0 !== E) {
                            if (r = ce(0, 255 & u[b - 1]), r && ve(!1), b++, D--, 0 === t.avail_out) {
                                return 0;
                            }
                        } else {
                            E = 1, b++, D--;
                        }
                    }
                    return 0 !== E && (r = ce(0, 255 & u[b - 1]), E = 0), ve(4 == e), 0 === t.avail_out ? 4 == e ? 2 : 0 : 4 == e ? 3 : 1;
                }(h);
            }
            if (2 != T && 3 != T || (r = $), 0 == T || 2 == T) {
                return 0 === t.avail_out && (o = -1), 0;
            }
            if (1 == T) {
                if (1 == h) {
                    ae(2, 3), se(F, j.static_ltree), le(), 1 + Q + 10 - te < 9 && (ae(2, 3), se(F, j.static_ltree), 
                    le()), Q = 7;
                } else if (pe(0, 0, !1), 3 == h) {
                    for (A = 0; A < p; A++) {
                        f[A] = 0;
                    }
                }
                if (t.flush_pending(), 0 === t.avail_out) {
                    return o = -1, 0;
                }
            }
        }
        return 4 != h ? 0 : 1;
    };
}

function q() {
    const e = this;
    e.next_in_index = 0, e.next_out_index = 0, e.avail_in = 0, e.total_in = 0, e.avail_out = 0, 
    e.total_out = 0;
}

q.prototype = {
    deflateInit(e, t) {
        const r = this;
        return r.dstate = new K, t || (t = 15), r.dstate.deflateInit(r, e, t);
    },
    deflate(e) {
        const t = this;
        return t.dstate ? t.dstate.deflate(t, e) : M;
    },
    deflateEnd() {
        const e = this;
        if (!e.dstate) {
            return M;
        }
        const t = e.dstate.deflateEnd();
        return e.dstate = null, t;
    },
    deflateParams(e, t) {
        const r = this;
        return r.dstate ? r.dstate.deflateParams(r, e, t) : M;
    },
    deflateSetDictionary(e, t) {
        const r = this;
        return r.dstate ? r.dstate.deflateSetDictionary(r, e, t) : M;
    },
    read_buf(e, t, r) {
        const n = this;
        let o = n.avail_in;
        return o > r && (o = r), 0 === o ? 0 : (n.avail_in -= o, e.set(n.next_in.subarray(n.next_in_index, n.next_in_index + o), t), 
        n.next_in_index += o, n.total_in += o, o);
    },
    flush_pending() {
        const e = this;
        let t = e.dstate.pending;
        t > e.avail_out && (t = e.avail_out), 0 !== t && (e.next_out.set(e.dstate.pending_buf.subarray(e.dstate.pending_out, e.dstate.pending_out + t), e.next_out_index), 
        e.next_out_index += t, e.dstate.pending_out += t, e.total_out += t, e.avail_out -= t, 
        e.dstate.pending -= t, 0 === e.dstate.pending && (e.dstate.pending_out = 0));
    }
};

const J = -2, X = -3, Z = -5, Y = [ 0, 1, 3, 7, 15, 31, 63, 127, 255, 511, 1023, 2047, 4095, 8191, 16383, 32767, 65535 ], Q = [ 96, 7, 256, 0, 8, 80, 0, 8, 16, 84, 8, 115, 82, 7, 31, 0, 8, 112, 0, 8, 48, 0, 9, 192, 80, 7, 10, 0, 8, 96, 0, 8, 32, 0, 9, 160, 0, 8, 0, 0, 8, 128, 0, 8, 64, 0, 9, 224, 80, 7, 6, 0, 8, 88, 0, 8, 24, 0, 9, 144, 83, 7, 59, 0, 8, 120, 0, 8, 56, 0, 9, 208, 81, 7, 17, 0, 8, 104, 0, 8, 40, 0, 9, 176, 0, 8, 8, 0, 8, 136, 0, 8, 72, 0, 9, 240, 80, 7, 4, 0, 8, 84, 0, 8, 20, 85, 8, 227, 83, 7, 43, 0, 8, 116, 0, 8, 52, 0, 9, 200, 81, 7, 13, 0, 8, 100, 0, 8, 36, 0, 9, 168, 0, 8, 4, 0, 8, 132, 0, 8, 68, 0, 9, 232, 80, 7, 8, 0, 8, 92, 0, 8, 28, 0, 9, 152, 84, 7, 83, 0, 8, 124, 0, 8, 60, 0, 9, 216, 82, 7, 23, 0, 8, 108, 0, 8, 44, 0, 9, 184, 0, 8, 12, 0, 8, 140, 0, 8, 76, 0, 9, 248, 80, 7, 3, 0, 8, 82, 0, 8, 18, 85, 8, 163, 83, 7, 35, 0, 8, 114, 0, 8, 50, 0, 9, 196, 81, 7, 11, 0, 8, 98, 0, 8, 34, 0, 9, 164, 0, 8, 2, 0, 8, 130, 0, 8, 66, 0, 9, 228, 80, 7, 7, 0, 8, 90, 0, 8, 26, 0, 9, 148, 84, 7, 67, 0, 8, 122, 0, 8, 58, 0, 9, 212, 82, 7, 19, 0, 8, 106, 0, 8, 42, 0, 9, 180, 0, 8, 10, 0, 8, 138, 0, 8, 74, 0, 9, 244, 80, 7, 5, 0, 8, 86, 0, 8, 22, 192, 8, 0, 83, 7, 51, 0, 8, 118, 0, 8, 54, 0, 9, 204, 81, 7, 15, 0, 8, 102, 0, 8, 38, 0, 9, 172, 0, 8, 6, 0, 8, 134, 0, 8, 70, 0, 9, 236, 80, 7, 9, 0, 8, 94, 0, 8, 30, 0, 9, 156, 84, 7, 99, 0, 8, 126, 0, 8, 62, 0, 9, 220, 82, 7, 27, 0, 8, 110, 0, 8, 46, 0, 9, 188, 0, 8, 14, 0, 8, 142, 0, 8, 78, 0, 9, 252, 96, 7, 256, 0, 8, 81, 0, 8, 17, 85, 8, 131, 82, 7, 31, 0, 8, 113, 0, 8, 49, 0, 9, 194, 80, 7, 10, 0, 8, 97, 0, 8, 33, 0, 9, 162, 0, 8, 1, 0, 8, 129, 0, 8, 65, 0, 9, 226, 80, 7, 6, 0, 8, 89, 0, 8, 25, 0, 9, 146, 83, 7, 59, 0, 8, 121, 0, 8, 57, 0, 9, 210, 81, 7, 17, 0, 8, 105, 0, 8, 41, 0, 9, 178, 0, 8, 9, 0, 8, 137, 0, 8, 73, 0, 9, 242, 80, 7, 4, 0, 8, 85, 0, 8, 21, 80, 8, 258, 83, 7, 43, 0, 8, 117, 0, 8, 53, 0, 9, 202, 81, 7, 13, 0, 8, 101, 0, 8, 37, 0, 9, 170, 0, 8, 5, 0, 8, 133, 0, 8, 69, 0, 9, 234, 80, 7, 8, 0, 8, 93, 0, 8, 29, 0, 9, 154, 84, 7, 83, 0, 8, 125, 0, 8, 61, 0, 9, 218, 82, 7, 23, 0, 8, 109, 0, 8, 45, 0, 9, 186, 0, 8, 13, 0, 8, 141, 0, 8, 77, 0, 9, 250, 80, 7, 3, 0, 8, 83, 0, 8, 19, 85, 8, 195, 83, 7, 35, 0, 8, 115, 0, 8, 51, 0, 9, 198, 81, 7, 11, 0, 8, 99, 0, 8, 35, 0, 9, 166, 0, 8, 3, 0, 8, 131, 0, 8, 67, 0, 9, 230, 80, 7, 7, 0, 8, 91, 0, 8, 27, 0, 9, 150, 84, 7, 67, 0, 8, 123, 0, 8, 59, 0, 9, 214, 82, 7, 19, 0, 8, 107, 0, 8, 43, 0, 9, 182, 0, 8, 11, 0, 8, 139, 0, 8, 75, 0, 9, 246, 80, 7, 5, 0, 8, 87, 0, 8, 23, 192, 8, 0, 83, 7, 51, 0, 8, 119, 0, 8, 55, 0, 9, 206, 81, 7, 15, 0, 8, 103, 0, 8, 39, 0, 9, 174, 0, 8, 7, 0, 8, 135, 0, 8, 71, 0, 9, 238, 80, 7, 9, 0, 8, 95, 0, 8, 31, 0, 9, 158, 84, 7, 99, 0, 8, 127, 0, 8, 63, 0, 9, 222, 82, 7, 27, 0, 8, 111, 0, 8, 47, 0, 9, 190, 0, 8, 15, 0, 8, 143, 0, 8, 79, 0, 9, 254, 96, 7, 256, 0, 8, 80, 0, 8, 16, 84, 8, 115, 82, 7, 31, 0, 8, 112, 0, 8, 48, 0, 9, 193, 80, 7, 10, 0, 8, 96, 0, 8, 32, 0, 9, 161, 0, 8, 0, 0, 8, 128, 0, 8, 64, 0, 9, 225, 80, 7, 6, 0, 8, 88, 0, 8, 24, 0, 9, 145, 83, 7, 59, 0, 8, 120, 0, 8, 56, 0, 9, 209, 81, 7, 17, 0, 8, 104, 0, 8, 40, 0, 9, 177, 0, 8, 8, 0, 8, 136, 0, 8, 72, 0, 9, 241, 80, 7, 4, 0, 8, 84, 0, 8, 20, 85, 8, 227, 83, 7, 43, 0, 8, 116, 0, 8, 52, 0, 9, 201, 81, 7, 13, 0, 8, 100, 0, 8, 36, 0, 9, 169, 0, 8, 4, 0, 8, 132, 0, 8, 68, 0, 9, 233, 80, 7, 8, 0, 8, 92, 0, 8, 28, 0, 9, 153, 84, 7, 83, 0, 8, 124, 0, 8, 60, 0, 9, 217, 82, 7, 23, 0, 8, 108, 0, 8, 44, 0, 9, 185, 0, 8, 12, 0, 8, 140, 0, 8, 76, 0, 9, 249, 80, 7, 3, 0, 8, 82, 0, 8, 18, 85, 8, 163, 83, 7, 35, 0, 8, 114, 0, 8, 50, 0, 9, 197, 81, 7, 11, 0, 8, 98, 0, 8, 34, 0, 9, 165, 0, 8, 2, 0, 8, 130, 0, 8, 66, 0, 9, 229, 80, 7, 7, 0, 8, 90, 0, 8, 26, 0, 9, 149, 84, 7, 67, 0, 8, 122, 0, 8, 58, 0, 9, 213, 82, 7, 19, 0, 8, 106, 0, 8, 42, 0, 9, 181, 0, 8, 10, 0, 8, 138, 0, 8, 74, 0, 9, 245, 80, 7, 5, 0, 8, 86, 0, 8, 22, 192, 8, 0, 83, 7, 51, 0, 8, 118, 0, 8, 54, 0, 9, 205, 81, 7, 15, 0, 8, 102, 0, 8, 38, 0, 9, 173, 0, 8, 6, 0, 8, 134, 0, 8, 70, 0, 9, 237, 80, 7, 9, 0, 8, 94, 0, 8, 30, 0, 9, 157, 84, 7, 99, 0, 8, 126, 0, 8, 62, 0, 9, 221, 82, 7, 27, 0, 8, 110, 0, 8, 46, 0, 9, 189, 0, 8, 14, 0, 8, 142, 0, 8, 78, 0, 9, 253, 96, 7, 256, 0, 8, 81, 0, 8, 17, 85, 8, 131, 82, 7, 31, 0, 8, 113, 0, 8, 49, 0, 9, 195, 80, 7, 10, 0, 8, 97, 0, 8, 33, 0, 9, 163, 0, 8, 1, 0, 8, 129, 0, 8, 65, 0, 9, 227, 80, 7, 6, 0, 8, 89, 0, 8, 25, 0, 9, 147, 83, 7, 59, 0, 8, 121, 0, 8, 57, 0, 9, 211, 81, 7, 17, 0, 8, 105, 0, 8, 41, 0, 9, 179, 0, 8, 9, 0, 8, 137, 0, 8, 73, 0, 9, 243, 80, 7, 4, 0, 8, 85, 0, 8, 21, 80, 8, 258, 83, 7, 43, 0, 8, 117, 0, 8, 53, 0, 9, 203, 81, 7, 13, 0, 8, 101, 0, 8, 37, 0, 9, 171, 0, 8, 5, 0, 8, 133, 0, 8, 69, 0, 9, 235, 80, 7, 8, 0, 8, 93, 0, 8, 29, 0, 9, 155, 84, 7, 83, 0, 8, 125, 0, 8, 61, 0, 9, 219, 82, 7, 23, 0, 8, 109, 0, 8, 45, 0, 9, 187, 0, 8, 13, 0, 8, 141, 0, 8, 77, 0, 9, 251, 80, 7, 3, 0, 8, 83, 0, 8, 19, 85, 8, 195, 83, 7, 35, 0, 8, 115, 0, 8, 51, 0, 9, 199, 81, 7, 11, 0, 8, 99, 0, 8, 35, 0, 9, 167, 0, 8, 3, 0, 8, 131, 0, 8, 67, 0, 9, 231, 80, 7, 7, 0, 8, 91, 0, 8, 27, 0, 9, 151, 84, 7, 67, 0, 8, 123, 0, 8, 59, 0, 9, 215, 82, 7, 19, 0, 8, 107, 0, 8, 43, 0, 9, 183, 0, 8, 11, 0, 8, 139, 0, 8, 75, 0, 9, 247, 80, 7, 5, 0, 8, 87, 0, 8, 23, 192, 8, 0, 83, 7, 51, 0, 8, 119, 0, 8, 55, 0, 9, 207, 81, 7, 15, 0, 8, 103, 0, 8, 39, 0, 9, 175, 0, 8, 7, 0, 8, 135, 0, 8, 71, 0, 9, 239, 80, 7, 9, 0, 8, 95, 0, 8, 31, 0, 9, 159, 84, 7, 99, 0, 8, 127, 0, 8, 63, 0, 9, 223, 82, 7, 27, 0, 8, 111, 0, 8, 47, 0, 9, 191, 0, 8, 15, 0, 8, 143, 0, 8, 79, 0, 9, 255 ], ee = [ 80, 5, 1, 87, 5, 257, 83, 5, 17, 91, 5, 4097, 81, 5, 5, 89, 5, 1025, 85, 5, 65, 93, 5, 16385, 80, 5, 3, 88, 5, 513, 84, 5, 33, 92, 5, 8193, 82, 5, 9, 90, 5, 2049, 86, 5, 129, 192, 5, 24577, 80, 5, 2, 87, 5, 385, 83, 5, 25, 91, 5, 6145, 81, 5, 7, 89, 5, 1537, 85, 5, 97, 93, 5, 24577, 80, 5, 4, 88, 5, 769, 84, 5, 49, 92, 5, 12289, 82, 5, 13, 90, 5, 3073, 86, 5, 193, 192, 5, 24577 ], te = [ 3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195, 227, 258, 0, 0 ], re = [ 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0, 112, 112 ], ne = [ 1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577 ], oe = [ 0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13 ], ie = 15;

function ae() {
    let e, t, r, n, o, i;
    function a(e, t, a, s, u, l, c, f, d, p, h) {
        let v, g, m, y, _, E, b, w, D, S, A, O, C, x, F;
        S = 0, _ = a;
        do {
            r[e[t + S]]++, S++, _--;
        } while (0 !== _);
        if (r[0] == a) {
            return c[0] = -1, f[0] = 0, 0;
        }
        for (w = f[0], E = 1; E <= ie && 0 === r[E]; E++) {}
        for (b = E, w < E && (w = E), _ = ie; 0 !== _ && 0 === r[_]; _--) {}
        for (m = _, w > _ && (w = _), f[0] = w, x = 1 << E; E < _; E++, x <<= 1) {
            if ((x -= r[E]) < 0) {
                return X;
            }
        }
        if ((x -= r[_]) < 0) {
            return X;
        }
        for (r[_] += x, i[1] = E = 0, S = 1, C = 2; 0 !== --_; ) {
            i[C] = E += r[S], C++, S++;
        }
        _ = 0, S = 0;
        do {
            0 !== (E = e[t + S]) && (h[i[E]++] = _), S++;
        } while (++_ < a);
        for (a = i[m], i[0] = _ = 0, S = 0, y = -1, O = -w, o[0] = 0, A = 0, F = 0; b <= m; b++) {
            for (v = r[b]; 0 !== v--; ) {
                for (;b > O + w; ) {
                    if (y++, O += w, F = m - O, F = F > w ? w : F, (g = 1 << (E = b - O)) > v + 1 && (g -= v + 1, 
                    C = b, E < F)) {
                        for (;++E < F && !((g <<= 1) <= r[++C]); ) {
                            g -= r[C];
                        }
                    }
                    if (F = 1 << E, p[0] + F > 1440) {
                        return X;
                    }
                    o[y] = A = p[0], p[0] += F, 0 !== y ? (i[y] = _, n[0] = E, n[1] = w, E = _ >>> O - w, 
                    n[2] = A - o[y - 1] - E, d.set(n, 3 * (o[y - 1] + E))) : c[0] = A;
                }
                for (n[1] = b - O, S >= a ? n[0] = 192 : h[S] < s ? (n[0] = h[S] < 256 ? 0 : 96, 
                n[2] = h[S++]) : (n[0] = l[h[S] - s] + 16 + 64, n[2] = u[h[S++] - s]), g = 1 << b - O, 
                E = _ >>> O; E < F; E += g) {
                    d.set(n, 3 * (A + E));
                }
                for (E = 1 << b - 1; 0 !== (_ & E); E >>>= 1) {
                    _ ^= E;
                }
                for (_ ^= E, D = (1 << O) - 1; (_ & D) != i[y]; ) {
                    y--, O -= w, D = (1 << O) - 1;
                }
            }
        }
        return 0 !== x && 1 != m ? Z : 0;
    }
    function s(a) {
        let s;
        for (e || (e = [], t = [], r = new Int32Array(16), n = [], o = new Int32Array(ie), 
        i = new Int32Array(16)), t.length < a && (t = []), s = 0; s < a; s++) {
            t[s] = 0;
        }
        for (s = 0; s < 16; s++) {
            r[s] = 0;
        }
        for (s = 0; s < 3; s++) {
            n[s] = 0;
        }
        o.set(r.subarray(0, ie), 0), i.set(r.subarray(0, 16), 0);
    }
    this.inflate_trees_bits = function(r, n, o, i, u) {
        let l;
        return s(19), e[0] = 0, l = a(r, 0, 19, 19, null, null, o, n, i, e, t), l == X ? u.msg = "oversubscribed dynamic bit lengths tree" : l != Z && 0 !== n[0] || (u.msg = "incomplete dynamic bit lengths tree", 
        l = X), l;
    }, this.inflate_trees_dynamic = function(r, n, o, i, u, l, c, f, d) {
        let p;
        return s(288), e[0] = 0, p = a(o, 0, r, 257, te, re, l, i, f, e, t), 0 != p || 0 === i[0] ? (p == X ? d.msg = "oversubscribed literal/length tree" : -4 != p && (d.msg = "incomplete literal/length tree", 
        p = X), p) : (s(288), p = a(o, r, n, 0, ne, oe, c, u, f, e, t), 0 != p || 0 === u[0] && r > 257 ? (p == X ? d.msg = "oversubscribed distance tree" : p == Z ? (d.msg = "incomplete distance tree", 
        p = X) : -4 != p && (d.msg = "empty distance tree with lengths", p = X), p) : 0);
    };
}

ae.inflate_trees_fixed = function(e, t, r, n) {
    return e[0] = 9, t[0] = 5, r[0] = Q, n[0] = ee, 0;
};

function se() {
    const e = this;
    let t, r, n, o, i = 0, a = 0, s = 0, u = 0, l = 0, c = 0, f = 0, d = 0, p = 0, h = 0;
    function v(e, t, r, n, o, i, a, s) {
        let u, l, c, f, d, p, h, v, g, m, y, _, E, b, w, D;
        h = s.next_in_index, v = s.avail_in, d = a.bitb, p = a.bitk, g = a.write, m = g < a.read ? a.read - g - 1 : a.end - g, 
        y = Y[e], _ = Y[t];
        do {
            for (;p < 20; ) {
                v--, d |= (255 & s.read_byte(h++)) << p, p += 8;
            }
            if (u = d & y, l = r, c = n, D = 3 * (c + u), 0 !== (f = l[D])) {
                for (;;) {
                    if (d >>= l[D + 1], p -= l[D + 1], 16 & f) {
                        for (f &= 15, E = l[D + 2] + (d & Y[f]), d >>= f, p -= f; p < 15; ) {
                            v--, d |= (255 & s.read_byte(h++)) << p, p += 8;
                        }
                        for (u = d & _, l = o, c = i, D = 3 * (c + u), f = l[D]; ;) {
                            if (d >>= l[D + 1], p -= l[D + 1], 16 & f) {
                                for (f &= 15; p < f; ) {
                                    v--, d |= (255 & s.read_byte(h++)) << p, p += 8;
                                }
                                if (b = l[D + 2] + (d & Y[f]), d >>= f, p -= f, m -= E, g >= b) {
                                    w = g - b, g - w > 0 && 2 > g - w ? (a.win[g++] = a.win[w++], a.win[g++] = a.win[w++], 
                                    E -= 2) : (a.win.set(a.win.subarray(w, w + 2), g), g += 2, w += 2, E -= 2);
                                } else {
                                    w = g - b;
                                    do {
                                        w += a.end;
                                    } while (w < 0);
                                    if (f = a.end - w, E > f) {
                                        if (E -= f, g - w > 0 && f > g - w) {
                                            do {
                                                a.win[g++] = a.win[w++];
                                            } while (0 !== --f);
                                        } else {
                                            a.win.set(a.win.subarray(w, w + f), g), g += f, w += f, f = 0;
                                        }
                                        w = 0;
                                    }
                                }
                                if (g - w > 0 && E > g - w) {
                                    do {
                                        a.win[g++] = a.win[w++];
                                    } while (0 !== --E);
                                } else {
                                    a.win.set(a.win.subarray(w, w + E), g), g += E, w += E, E = 0;
                                }
                                break;
                            }
                            if (64 & f) {
                                return s.msg = "invalid distance code", E = s.avail_in - v, E = p >> 3 < E ? p >> 3 : E, 
                                v += E, h -= E, p -= E << 3, a.bitb = d, a.bitk = p, s.avail_in = v, s.total_in += h - s.next_in_index, 
                                s.next_in_index = h, a.write = g, X;
                            }
                            u += l[D + 2], u += d & Y[f], D = 3 * (c + u), f = l[D];
                        }
                        break;
                    }
                    if (64 & f) {
                        return 32 & f ? (E = s.avail_in - v, E = p >> 3 < E ? p >> 3 : E, v += E, h -= E, 
                        p -= E << 3, a.bitb = d, a.bitk = p, s.avail_in = v, s.total_in += h - s.next_in_index, 
                        s.next_in_index = h, a.write = g, 1) : (s.msg = "invalid literal/length code", E = s.avail_in - v, 
                        E = p >> 3 < E ? p >> 3 : E, v += E, h -= E, p -= E << 3, a.bitb = d, a.bitk = p, 
                        s.avail_in = v, s.total_in += h - s.next_in_index, s.next_in_index = h, a.write = g, 
                        X);
                    }
                    if (u += l[D + 2], u += d & Y[f], D = 3 * (c + u), 0 === (f = l[D])) {
                        d >>= l[D + 1], p -= l[D + 1], a.win[g++] = l[D + 2], m--;
                        break;
                    }
                }
            } else {
                d >>= l[D + 1], p -= l[D + 1], a.win[g++] = l[D + 2], m--;
            }
        } while (m >= 258 && v >= 10);
        return E = s.avail_in - v, E = p >> 3 < E ? p >> 3 : E, v += E, h -= E, p -= E << 3, 
        a.bitb = d, a.bitk = p, s.avail_in = v, s.total_in += h - s.next_in_index, s.next_in_index = h, 
        a.write = g, 0;
    }
    e.init = function(e, i, a, s, u, l) {
        t = 0, f = e, d = i, n = a, p = s, o = u, h = l, r = null;
    }, e.proc = function(e, g, m) {
        let y, _, E, b, w, D, S, A = 0, O = 0, C = 0;
        for (C = g.next_in_index, b = g.avail_in, A = e.bitb, O = e.bitk, w = e.write, D = w < e.read ? e.read - w - 1 : e.end - w; ;) {
            switch (t) {
              case 0:
                if (D >= 258 && b >= 10 && (e.bitb = A, e.bitk = O, g.avail_in = b, g.total_in += C - g.next_in_index, 
                g.next_in_index = C, e.write = w, m = v(f, d, n, p, o, h, e, g), C = g.next_in_index, 
                b = g.avail_in, A = e.bitb, O = e.bitk, w = e.write, D = w < e.read ? e.read - w - 1 : e.end - w, 
                0 != m)) {
                    t = 1 == m ? 7 : 9;
                    break;
                }
                s = f, r = n, a = p, t = 1;

              case 1:
                for (y = s; O < y; ) {
                    if (0 === b) {
                        return e.bitb = A, e.bitk = O, g.avail_in = b, g.total_in += C - g.next_in_index, 
                        g.next_in_index = C, e.write = w, e.inflate_flush(g, m);
                    }
                    m = 0, b--, A |= (255 & g.read_byte(C++)) << O, O += 8;
                }
                if (_ = 3 * (a + (A & Y[y])), A >>>= r[_ + 1], O -= r[_ + 1], E = r[_], 0 === E) {
                    u = r[_ + 2], t = 6;
                    break;
                }
                if (16 & E) {
                    l = 15 & E, i = r[_ + 2], t = 2;
                    break;
                }
                if (!(64 & E)) {
                    s = E, a = _ / 3 + r[_ + 2];
                    break;
                }
                if (32 & E) {
                    t = 7;
                    break;
                }
                return t = 9, g.msg = "invalid literal/length code", m = X, e.bitb = A, e.bitk = O, 
                g.avail_in = b, g.total_in += C - g.next_in_index, g.next_in_index = C, e.write = w, 
                e.inflate_flush(g, m);

              case 2:
                for (y = l; O < y; ) {
                    if (0 === b) {
                        return e.bitb = A, e.bitk = O, g.avail_in = b, g.total_in += C - g.next_in_index, 
                        g.next_in_index = C, e.write = w, e.inflate_flush(g, m);
                    }
                    m = 0, b--, A |= (255 & g.read_byte(C++)) << O, O += 8;
                }
                i += A & Y[y], A >>= y, O -= y, s = d, r = o, a = h, t = 3;

              case 3:
                for (y = s; O < y; ) {
                    if (0 === b) {
                        return e.bitb = A, e.bitk = O, g.avail_in = b, g.total_in += C - g.next_in_index, 
                        g.next_in_index = C, e.write = w, e.inflate_flush(g, m);
                    }
                    m = 0, b--, A |= (255 & g.read_byte(C++)) << O, O += 8;
                }
                if (_ = 3 * (a + (A & Y[y])), A >>= r[_ + 1], O -= r[_ + 1], E = r[_], 16 & E) {
                    l = 15 & E, c = r[_ + 2], t = 4;
                    break;
                }
                if (!(64 & E)) {
                    s = E, a = _ / 3 + r[_ + 2];
                    break;
                }
                return t = 9, g.msg = "invalid distance code", m = X, e.bitb = A, e.bitk = O, g.avail_in = b, 
                g.total_in += C - g.next_in_index, g.next_in_index = C, e.write = w, e.inflate_flush(g, m);

              case 4:
                for (y = l; O < y; ) {
                    if (0 === b) {
                        return e.bitb = A, e.bitk = O, g.avail_in = b, g.total_in += C - g.next_in_index, 
                        g.next_in_index = C, e.write = w, e.inflate_flush(g, m);
                    }
                    m = 0, b--, A |= (255 & g.read_byte(C++)) << O, O += 8;
                }
                c += A & Y[y], A >>= y, O -= y, t = 5;

              case 5:
                for (S = w - c; S < 0; ) {
                    S += e.end;
                }
                for (;0 !== i; ) {
                    if (0 === D && (w == e.end && 0 !== e.read && (w = 0, D = w < e.read ? e.read - w - 1 : e.end - w), 
                    0 === D && (e.write = w, m = e.inflate_flush(g, m), w = e.write, D = w < e.read ? e.read - w - 1 : e.end - w, 
                    w == e.end && 0 !== e.read && (w = 0, D = w < e.read ? e.read - w - 1 : e.end - w), 
                    0 === D))) {
                        return e.bitb = A, e.bitk = O, g.avail_in = b, g.total_in += C - g.next_in_index, 
                        g.next_in_index = C, e.write = w, e.inflate_flush(g, m);
                    }
                    e.win[w++] = e.win[S++], D--, S == e.end && (S = 0), i--;
                }
                t = 0;
                break;

              case 6:
                if (0 === D && (w == e.end && 0 !== e.read && (w = 0, D = w < e.read ? e.read - w - 1 : e.end - w), 
                0 === D && (e.write = w, m = e.inflate_flush(g, m), w = e.write, D = w < e.read ? e.read - w - 1 : e.end - w, 
                w == e.end && 0 !== e.read && (w = 0, D = w < e.read ? e.read - w - 1 : e.end - w), 
                0 === D))) {
                    return e.bitb = A, e.bitk = O, g.avail_in = b, g.total_in += C - g.next_in_index, 
                    g.next_in_index = C, e.write = w, e.inflate_flush(g, m);
                }
                m = 0, e.win[w++] = u, D--, t = 0;
                break;

              case 7:
                if (O > 7 && (O -= 8, b++, C--), e.write = w, m = e.inflate_flush(g, m), w = e.write, 
                D = w < e.read ? e.read - w - 1 : e.end - w, e.read != e.write) {
                    return e.bitb = A, e.bitk = O, g.avail_in = b, g.total_in += C - g.next_in_index, 
                    g.next_in_index = C, e.write = w, e.inflate_flush(g, m);
                }
                t = 8;

              case 8:
                return m = 1, e.bitb = A, e.bitk = O, g.avail_in = b, g.total_in += C - g.next_in_index, 
                g.next_in_index = C, e.write = w, e.inflate_flush(g, m);

              case 9:
                return m = X, e.bitb = A, e.bitk = O, g.avail_in = b, g.total_in += C - g.next_in_index, 
                g.next_in_index = C, e.write = w, e.inflate_flush(g, m);

              default:
                return m = J, e.bitb = A, e.bitk = O, g.avail_in = b, g.total_in += C - g.next_in_index, 
                g.next_in_index = C, e.write = w, e.inflate_flush(g, m);
            }
        }
    }, e.free = function() {};
}

const ue = [ 16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15 ];

function le(e, t) {
    const r = this;
    let n, o = 0, i = 0, a = 0, s = 0;
    const u = [ 0 ], l = [ 0 ], c = new se;
    let f = 0, d = new Int32Array(4320);
    const p = new ae;
    r.bitk = 0, r.bitb = 0, r.win = new Uint8Array(t), r.end = t, r.read = 0, r.write = 0, 
    r.reset = function(e, t) {
        t && (t[0] = 0), 6 == o && c.free(e), o = 0, r.bitk = 0, r.bitb = 0, r.read = r.write = 0;
    }, r.reset(e, null), r.inflate_flush = function(e, t) {
        let n, o, i;
        return o = e.next_out_index, i = r.read, n = (i <= r.write ? r.write : r.end) - i, 
        n > e.avail_out && (n = e.avail_out), 0 !== n && t == Z && (t = 0), e.avail_out -= n, 
        e.total_out += n, e.next_out.set(r.win.subarray(i, i + n), o), o += n, i += n, i == r.end && (i = 0, 
        r.write == r.end && (r.write = 0), n = r.write - i, n > e.avail_out && (n = e.avail_out), 
        0 !== n && t == Z && (t = 0), e.avail_out -= n, e.total_out += n, e.next_out.set(r.win.subarray(i, i + n), o), 
        o += n, i += n), e.next_out_index = o, r.read = i, t;
    }, r.proc = function(e, t) {
        let h, v, g, m, y, _, E, b;
        for (m = e.next_in_index, y = e.avail_in, v = r.bitb, g = r.bitk, _ = r.write, E = _ < r.read ? r.read - _ - 1 : r.end - _; ;) {
            let w, D, S, A, O, C, x, F;
            switch (o) {
              case 0:
                for (;g < 3; ) {
                    if (0 === y) {
                        return r.bitb = v, r.bitk = g, e.avail_in = y, e.total_in += m - e.next_in_index, 
                        e.next_in_index = m, r.write = _, r.inflate_flush(e, t);
                    }
                    t = 0, y--, v |= (255 & e.read_byte(m++)) << g, g += 8;
                }
                switch (h = 7 & v, f = 1 & h, h >>> 1) {
                  case 0:
                    v >>>= 3, g -= 3, h = 7 & g, v >>>= h, g -= h, o = 1;
                    break;

                  case 1:
                    w = [], D = [], S = [ [] ], A = [ [] ], ae.inflate_trees_fixed(w, D, S, A), c.init(w[0], D[0], S[0], 0, A[0], 0), 
                    v >>>= 3, g -= 3, o = 6;
                    break;

                  case 2:
                    v >>>= 3, g -= 3, o = 3;
                    break;

                  case 3:
                    return v >>>= 3, g -= 3, o = 9, e.msg = "invalid block type", t = X, r.bitb = v, 
                    r.bitk = g, e.avail_in = y, e.total_in += m - e.next_in_index, e.next_in_index = m, 
                    r.write = _, r.inflate_flush(e, t);
                }
                break;

              case 1:
                for (;g < 32; ) {
                    if (0 === y) {
                        return r.bitb = v, r.bitk = g, e.avail_in = y, e.total_in += m - e.next_in_index, 
                        e.next_in_index = m, r.write = _, r.inflate_flush(e, t);
                    }
                    t = 0, y--, v |= (255 & e.read_byte(m++)) << g, g += 8;
                }
                if ((~v >>> 16 & 65535) != (65535 & v)) {
                    return o = 9, e.msg = "invalid stored block lengths", t = X, r.bitb = v, r.bitk = g, 
                    e.avail_in = y, e.total_in += m - e.next_in_index, e.next_in_index = m, r.write = _, 
                    r.inflate_flush(e, t);
                }
                i = 65535 & v, v = g = 0, o = 0 !== i ? 2 : 0 !== f ? 7 : 0;
                break;

              case 2:
                if (0 === y) {
                    return r.bitb = v, r.bitk = g, e.avail_in = y, e.total_in += m - e.next_in_index, 
                    e.next_in_index = m, r.write = _, r.inflate_flush(e, t);
                }
                if (0 === E && (_ == r.end && 0 !== r.read && (_ = 0, E = _ < r.read ? r.read - _ - 1 : r.end - _), 
                0 === E && (r.write = _, t = r.inflate_flush(e, t), _ = r.write, E = _ < r.read ? r.read - _ - 1 : r.end - _, 
                _ == r.end && 0 !== r.read && (_ = 0, E = _ < r.read ? r.read - _ - 1 : r.end - _), 
                0 === E))) {
                    return r.bitb = v, r.bitk = g, e.avail_in = y, e.total_in += m - e.next_in_index, 
                    e.next_in_index = m, r.write = _, r.inflate_flush(e, t);
                }
                if (t = 0, h = i, h > y && (h = y), h > E && (h = E), r.win.set(e.read_buf(m, h), _), 
                m += h, y -= h, _ += h, E -= h, 0 !== (i -= h)) {
                    break;
                }
                o = 0 !== f ? 7 : 0;
                break;

              case 3:
                for (;g < 14; ) {
                    if (0 === y) {
                        return r.bitb = v, r.bitk = g, e.avail_in = y, e.total_in += m - e.next_in_index, 
                        e.next_in_index = m, r.write = _, r.inflate_flush(e, t);
                    }
                    t = 0, y--, v |= (255 & e.read_byte(m++)) << g, g += 8;
                }
                if (a = h = 16383 & v, (31 & h) > 29 || (h >> 5 & 31) > 29) {
                    return o = 9, e.msg = "too many length or distance symbols", t = X, r.bitb = v, 
                    r.bitk = g, e.avail_in = y, e.total_in += m - e.next_in_index, e.next_in_index = m, 
                    r.write = _, r.inflate_flush(e, t);
                }
                if (h = 258 + (31 & h) + (h >> 5 & 31), !n || n.length < h) {
                    n = [];
                } else {
                    for (b = 0; b < h; b++) {
                        n[b] = 0;
                    }
                }
                v >>>= 14, g -= 14, s = 0, o = 4;

              case 4:
                for (;s < 4 + (a >>> 10); ) {
                    for (;g < 3; ) {
                        if (0 === y) {
                            return r.bitb = v, r.bitk = g, e.avail_in = y, e.total_in += m - e.next_in_index, 
                            e.next_in_index = m, r.write = _, r.inflate_flush(e, t);
                        }
                        t = 0, y--, v |= (255 & e.read_byte(m++)) << g, g += 8;
                    }
                    n[ue[s++]] = 7 & v, v >>>= 3, g -= 3;
                }
                for (;s < 19; ) {
                    n[ue[s++]] = 0;
                }
                if (u[0] = 7, h = p.inflate_trees_bits(n, u, l, d, e), 0 != h) {
                    return (t = h) == X && (n = null, o = 9), r.bitb = v, r.bitk = g, e.avail_in = y, 
                    e.total_in += m - e.next_in_index, e.next_in_index = m, r.write = _, r.inflate_flush(e, t);
                }
                s = 0, o = 5;

              case 5:
                for (;h = a, !(s >= 258 + (31 & h) + (h >> 5 & 31)); ) {
                    let i, c;
                    for (h = u[0]; g < h; ) {
                        if (0 === y) {
                            return r.bitb = v, r.bitk = g, e.avail_in = y, e.total_in += m - e.next_in_index, 
                            e.next_in_index = m, r.write = _, r.inflate_flush(e, t);
                        }
                        t = 0, y--, v |= (255 & e.read_byte(m++)) << g, g += 8;
                    }
                    if (h = d[3 * (l[0] + (v & Y[h])) + 1], c = d[3 * (l[0] + (v & Y[h])) + 2], c < 16) {
                        v >>>= h, g -= h, n[s++] = c;
                    } else {
                        for (b = 18 == c ? 7 : c - 14, i = 18 == c ? 11 : 3; g < h + b; ) {
                            if (0 === y) {
                                return r.bitb = v, r.bitk = g, e.avail_in = y, e.total_in += m - e.next_in_index, 
                                e.next_in_index = m, r.write = _, r.inflate_flush(e, t);
                            }
                            t = 0, y--, v |= (255 & e.read_byte(m++)) << g, g += 8;
                        }
                        if (v >>>= h, g -= h, i += v & Y[b], v >>>= b, g -= b, b = s, h = a, b + i > 258 + (31 & h) + (h >> 5 & 31) || 16 == c && b < 1) {
                            return n = null, o = 9, e.msg = "invalid bit length repeat", t = X, r.bitb = v, 
                            r.bitk = g, e.avail_in = y, e.total_in += m - e.next_in_index, e.next_in_index = m, 
                            r.write = _, r.inflate_flush(e, t);
                        }
                        c = 16 == c ? n[b - 1] : 0;
                        do {
                            n[b++] = c;
                        } while (0 !== --i);
                        s = b;
                    }
                }
                if (l[0] = -1, O = [], C = [], x = [], F = [], O[0] = 9, C[0] = 6, h = a, h = p.inflate_trees_dynamic(257 + (31 & h), 1 + (h >> 5 & 31), n, O, C, x, F, d, e), 
                0 != h) {
                    return h == X && (n = null, o = 9), t = h, r.bitb = v, r.bitk = g, e.avail_in = y, 
                    e.total_in += m - e.next_in_index, e.next_in_index = m, r.write = _, r.inflate_flush(e, t);
                }
                c.init(O[0], C[0], d, x[0], d, F[0]), o = 6;

              case 6:
                if (r.bitb = v, r.bitk = g, e.avail_in = y, e.total_in += m - e.next_in_index, e.next_in_index = m, 
                r.write = _, 1 != (t = c.proc(r, e, t))) {
                    return r.inflate_flush(e, t);
                }
                if (t = 0, c.free(e), m = e.next_in_index, y = e.avail_in, v = r.bitb, g = r.bitk, 
                _ = r.write, E = _ < r.read ? r.read - _ - 1 : r.end - _, 0 === f) {
                    o = 0;
                    break;
                }
                o = 7;

              case 7:
                if (r.write = _, t = r.inflate_flush(e, t), _ = r.write, E = _ < r.read ? r.read - _ - 1 : r.end - _, 
                r.read != r.write) {
                    return r.bitb = v, r.bitk = g, e.avail_in = y, e.total_in += m - e.next_in_index, 
                    e.next_in_index = m, r.write = _, r.inflate_flush(e, t);
                }
                o = 8;

              case 8:
                return t = 1, r.bitb = v, r.bitk = g, e.avail_in = y, e.total_in += m - e.next_in_index, 
                e.next_in_index = m, r.write = _, r.inflate_flush(e, t);

              case 9:
                return t = X, r.bitb = v, r.bitk = g, e.avail_in = y, e.total_in += m - e.next_in_index, 
                e.next_in_index = m, r.write = _, r.inflate_flush(e, t);

              default:
                return t = J, r.bitb = v, r.bitk = g, e.avail_in = y, e.total_in += m - e.next_in_index, 
                e.next_in_index = m, r.write = _, r.inflate_flush(e, t);
            }
        }
    }, r.free = function(e) {
        r.reset(e, null), r.win = null, d = null;
    }, r.set_dictionary = function(e, t, n) {
        r.win.set(e.subarray(t, t + n), 0), r.read = r.write = n;
    }, r.sync_point = function() {
        return 1 == o ? 1 : 0;
    };
}

const ce = 13, fe = [ 0, 0, 255, 255 ];

function de() {
    const e = this;
    function t(e) {
        return e && e.istate ? (e.total_in = e.total_out = 0, e.msg = null, e.istate.mode = 7, 
        e.istate.blocks.reset(e, null), 0) : J;
    }
    e.mode = 0, e.method = 0, e.was = [ 0 ], e.need = 0, e.marker = 0, e.wbits = 0, 
    e.inflateEnd = function(t) {
        return e.blocks && e.blocks.free(t), e.blocks = null, 0;
    }, e.inflateInit = function(r, n) {
        return r.msg = null, e.blocks = null, n < 8 || n > 15 ? (e.inflateEnd(r), J) : (e.wbits = n, 
        r.istate.blocks = new le(r, 1 << n), t(r), 0);
    }, e.inflate = function(e, t) {
        let r, n;
        if (!e || !e.istate || !e.next_in) {
            return J;
        }
        const o = e.istate;
        for (t = 4 == t ? Z : 0, r = Z; ;) {
            switch (o.mode) {
              case 0:
                if (0 === e.avail_in) {
                    return r;
                }
                if (r = t, e.avail_in--, e.total_in++, 8 != (15 & (o.method = e.read_byte(e.next_in_index++)))) {
                    o.mode = ce, e.msg = "unknown compression method", o.marker = 5;
                    break;
                }
                if (8 + (o.method >> 4) > o.wbits) {
                    o.mode = ce, e.msg = "invalid win size", o.marker = 5;
                    break;
                }
                o.mode = 1;

              case 1:
                if (0 === e.avail_in) {
                    return r;
                }
                if (r = t, e.avail_in--, e.total_in++, n = 255 & e.read_byte(e.next_in_index++), 
                ((o.method << 8) + n) % 31 != 0) {
                    o.mode = ce, e.msg = "incorrect header check", o.marker = 5;
                    break;
                }
                if (!(32 & n)) {
                    o.mode = 7;
                    break;
                }
                o.mode = 2;

              case 2:
                if (0 === e.avail_in) {
                    return r;
                }
                r = t, e.avail_in--, e.total_in++, o.need = (255 & e.read_byte(e.next_in_index++)) << 24 & 4278190080, 
                o.mode = 3;

              case 3:
                if (0 === e.avail_in) {
                    return r;
                }
                r = t, e.avail_in--, e.total_in++, o.need += (255 & e.read_byte(e.next_in_index++)) << 16 & 16711680, 
                o.mode = 4;

              case 4:
                if (0 === e.avail_in) {
                    return r;
                }
                r = t, e.avail_in--, e.total_in++, o.need += (255 & e.read_byte(e.next_in_index++)) << 8 & 65280, 
                o.mode = 5;

              case 5:
                return 0 === e.avail_in ? r : (r = t, e.avail_in--, e.total_in++, o.need += 255 & e.read_byte(e.next_in_index++), 
                o.mode = 6, 2);

              case 6:
                return o.mode = ce, e.msg = "need dictionary", o.marker = 0, J;

              case 7:
                if (r = o.blocks.proc(e, r), r == X) {
                    o.mode = ce, o.marker = 0;
                    break;
                }
                if (0 == r && (r = t), 1 != r) {
                    return r;
                }
                r = t, o.blocks.reset(e, o.was), o.mode = 12;

              case 12:
                return e.avail_in = 0, 1;

              case ce:
                return X;

              default:
                return J;
            }
        }
    }, e.inflateSetDictionary = function(e, t, r) {
        let n = 0, o = r;
        if (!e || !e.istate || 6 != e.istate.mode) {
            return J;
        }
        const i = e.istate;
        return o >= 1 << i.wbits && (o = (1 << i.wbits) - 1, n = r - o), i.blocks.set_dictionary(t, n, o), 
        i.mode = 7, 0;
    }, e.inflateSync = function(e) {
        let r, n, o, i, a;
        if (!e || !e.istate) {
            return J;
        }
        const s = e.istate;
        if (s.mode != ce && (s.mode = ce, s.marker = 0), 0 === (r = e.avail_in)) {
            return Z;
        }
        for (n = e.next_in_index, o = s.marker; 0 !== r && o < 4; ) {
            e.read_byte(n) == fe[o] ? o++ : o = 0 !== e.read_byte(n) ? 0 : 4 - o, n++, r--;
        }
        return e.total_in += n - e.next_in_index, e.next_in_index = n, e.avail_in = r, s.marker = o, 
        4 != o ? X : (i = e.total_in, a = e.total_out, t(e), e.total_in = i, e.total_out = a, 
        s.mode = 7, 0);
    }, e.inflateSyncPoint = function(e) {
        return e && e.istate && e.istate.blocks ? e.istate.blocks.sync_point() : J;
    };
}

function pe() {}

pe.prototype = {
    inflateInit(e) {
        const t = this;
        return t.istate = new de, e || (e = 15), t.istate.inflateInit(t, e);
    },
    inflate(e) {
        const t = this;
        return t.istate ? t.istate.inflate(t, e) : J;
    },
    inflateEnd() {
        const e = this;
        if (!e.istate) {
            return J;
        }
        const t = e.istate.inflateEnd(e);
        return e.istate = null, t;
    },
    inflateSync() {
        const e = this;
        return e.istate ? e.istate.inflateSync(e) : J;
    },
    inflateSetDictionary(e, t) {
        const r = this;
        return r.istate ? r.istate.inflateSetDictionary(r, e, t) : J;
    },
    read_byte(e) {
        return this.next_in[e];
    },
    read_buf(e, t) {
        return this.next_in.subarray(e, e + t);
    }
};

const he = 4294967295, ve = 65535, ge = 67324752, me = 134695760, ye = me, _e = 33639248, Ee = 101010256, be = 101075792, we = 117853008, De = 22, Se = 39169, Ae = 21589, Oe = 6534, Ce = 2048, xe = "/", Fe = new Date(2107, 11, 31), Me = new Date(1980, 0, 1), Pe = void 0, Ie = "undefined", ke = "function";

class Re {
    constructor(e) {
        return class extends TransformStream {
            constructor(t, r) {
                const n = new e(r);
                super({
                    transform(e, t) {
                        t.enqueue(n.append(e));
                    },
                    flush(e) {
                        const t = n.flush();
                        t && e.enqueue(t);
                    }
                });
            }
        };
    }
}

let Te = 2;

try {
    typeof navigator != Ie && navigator.hardwareConcurrency && (Te = navigator.hardwareConcurrency);
} catch (e) {}

const je = {
    chunkSize: 524288,
    maxWorkers: Te,
    terminateWorkerTimeout: 5e3,
    useWebWorkers: !0,
    useCompressionStream: !0,
    workerScripts: Pe,
    CompressionStreamNative: typeof CompressionStream != Ie && CompressionStream,
    DecompressionStreamNative: typeof DecompressionStream != Ie && DecompressionStream
}, Le = Object.assign({}, je);

function Ne() {
    return Le;
}

function Be(e) {
    return Math.max(e.chunkSize, 64);
}

function Ue(e) {
    const {baseURL: t, chunkSize: r, maxWorkers: n, terminateWorkerTimeout: o, useCompressionStream: i, useWebWorkers: a, Deflate: s, Inflate: u, CompressionStream: l, DecompressionStream: c, workerScripts: f} = e;
    if (ze("baseURL", t), ze("chunkSize", r), ze("maxWorkers", n), ze("terminateWorkerTimeout", o), 
    ze("useCompressionStream", i), ze("useWebWorkers", a), s && (Le.CompressionStream = new Re(s)), 
    u && (Le.DecompressionStream = new Re(u)), ze("CompressionStream", l), ze("DecompressionStream", c), 
    f !== Pe) {
        const {deflate: e, inflate: t} = f;
        if ((e || t) && (Le.workerScripts || (Le.workerScripts = {})), e) {
            if (!Array.isArray(e)) {
                throw new Error("workerScripts.deflate must be an array");
            }
            Le.workerScripts.deflate = e;
        }
        if (t) {
            if (!Array.isArray(t)) {
                throw new Error("workerScripts.inflate must be an array");
            }
            Le.workerScripts.inflate = t;
        }
    }
}

function ze(e, t) {
    t !== Pe && (Le[e] = t);
}

const He = {
    application: {
        "andrew-inset": "ez",
        annodex: "anx",
        "atom+xml": "atom",
        "atomcat+xml": "atomcat",
        "atomserv+xml": "atomsrv",
        bbolin: "lin",
        "cu-seeme": "cu",
        "davmount+xml": "davmount",
        dsptype: "tsp",
        ecmascript: [ "es", "ecma" ],
        futuresplash: "spl",
        hta: "hta",
        "java-archive": "jar",
        "java-serialized-object": "ser",
        "java-vm": "class",
        m3g: "m3g",
        "mac-binhex40": "hqx",
        mathematica: [ "nb", "ma", "mb" ],
        msaccess: "mdb",
        msword: [ "doc", "dot", "wiz" ],
        mxf: "mxf",
        oda: "oda",
        ogg: "ogx",
        pdf: "pdf",
        "pgp-keys": "key",
        "pgp-signature": [ "asc", "sig" ],
        "pics-rules": "prf",
        postscript: [ "ps", "ai", "eps", "epsi", "epsf", "eps2", "eps3" ],
        rar: "rar",
        "rdf+xml": "rdf",
        "rss+xml": "rss",
        rtf: "rtf",
        "xhtml+xml": [ "xhtml", "xht" ],
        xml: [ "xml", "xsl", "xsd", "xpdl" ],
        "xspf+xml": "xspf",
        zip: "zip",
        "vnd.android.package-archive": "apk",
        "vnd.cinderella": "cdy",
        "vnd.google-earth.kml+xml": "kml",
        "vnd.google-earth.kmz": "kmz",
        "vnd.mozilla.xul+xml": "xul",
        "vnd.ms-excel": [ "xls", "xlb", "xlt", "xlm", "xla", "xlc", "xlw" ],
        "vnd.ms-pki.seccat": "cat",
        "vnd.ms-pki.stl": "stl",
        "vnd.ms-powerpoint": [ "ppt", "pps", "pot", "ppa", "pwz" ],
        "vnd.oasis.opendocument.chart": "odc",
        "vnd.oasis.opendocument.database": "odb",
        "vnd.oasis.opendocument.formula": "odf",
        "vnd.oasis.opendocument.graphics": "odg",
        "vnd.oasis.opendocument.graphics-template": "otg",
        "vnd.oasis.opendocument.image": "odi",
        "vnd.oasis.opendocument.presentation": "odp",
        "vnd.oasis.opendocument.presentation-template": "otp",
        "vnd.oasis.opendocument.spreadsheet": "ods",
        "vnd.oasis.opendocument.spreadsheet-template": "ots",
        "vnd.oasis.opendocument.text": "odt",
        "vnd.oasis.opendocument.text-master": [ "odm", "otm" ],
        "vnd.oasis.opendocument.text-template": "ott",
        "vnd.oasis.opendocument.text-web": "oth",
        "vnd.openxmlformats-officedocument.spreadsheetml.sheet": "xlsx",
        "vnd.openxmlformats-officedocument.spreadsheetml.template": "xltx",
        "vnd.openxmlformats-officedocument.presentationml.presentation": "pptx",
        "vnd.openxmlformats-officedocument.presentationml.slideshow": "ppsx",
        "vnd.openxmlformats-officedocument.presentationml.template": "potx",
        "vnd.openxmlformats-officedocument.wordprocessingml.document": "docx",
        "vnd.openxmlformats-officedocument.wordprocessingml.template": "dotx",
        "vnd.smaf": "mmf",
        "vnd.stardivision.calc": "sdc",
        "vnd.stardivision.chart": "sds",
        "vnd.stardivision.draw": "sda",
        "vnd.stardivision.impress": "sdd",
        "vnd.stardivision.math": [ "sdf", "smf" ],
        "vnd.stardivision.writer": [ "sdw", "vor" ],
        "vnd.stardivision.writer-global": "sgl",
        "vnd.sun.xml.calc": "sxc",
        "vnd.sun.xml.calc.template": "stc",
        "vnd.sun.xml.draw": "sxd",
        "vnd.sun.xml.draw.template": "std",
        "vnd.sun.xml.impress": "sxi",
        "vnd.sun.xml.impress.template": "sti",
        "vnd.sun.xml.math": "sxm",
        "vnd.sun.xml.writer": "sxw",
        "vnd.sun.xml.writer.global": "sxg",
        "vnd.sun.xml.writer.template": "stw",
        "vnd.symbian.install": [ "sis", "sisx" ],
        "vnd.visio": [ "vsd", "vst", "vss", "vsw", "vsdx", "vssx", "vstx", "vssm", "vstm" ],
        "vnd.wap.wbxml": "wbxml",
        "vnd.wap.wmlc": "wmlc",
        "vnd.wap.wmlscriptc": "wmlsc",
        "vnd.wordperfect": "wpd",
        "vnd.wordperfect5.1": "wp5",
        "x-123": "wk",
        "x-7z-compressed": "7z",
        "x-abiword": "abw",
        "x-apple-diskimage": "dmg",
        "x-bcpio": "bcpio",
        "x-bittorrent": "torrent",
        "x-cbr": [ "cbr", "cba", "cbt", "cb7" ],
        "x-cbz": "cbz",
        "x-cdf": [ "cdf", "cda" ],
        "x-cdlink": "vcd",
        "x-chess-pgn": "pgn",
        "x-cpio": "cpio",
        "x-csh": "csh",
        "x-director": [ "dir", "dxr", "cst", "cct", "cxt", "w3d", "fgd", "swa" ],
        "x-dms": "dms",
        "x-doom": "wad",
        "x-dvi": "dvi",
        "x-httpd-eruby": "rhtml",
        "x-font": "pcf.Z",
        "x-freemind": "mm",
        "x-gnumeric": "gnumeric",
        "x-go-sgf": "sgf",
        "x-graphing-calculator": "gcf",
        "x-gtar": [ "gtar", "taz" ],
        "x-hdf": "hdf",
        "x-httpd-php": [ "phtml", "pht", "php" ],
        "x-httpd-php-source": "phps",
        "x-httpd-php3": "php3",
        "x-httpd-php3-preprocessed": "php3p",
        "x-httpd-php4": "php4",
        "x-httpd-php5": "php5",
        "x-ica": "ica",
        "x-info": "info",
        "x-internet-signup": [ "ins", "isp" ],
        "x-iphone": "iii",
        "x-iso9660-image": "iso",
        "x-java-jnlp-file": "jnlp",
        "x-jmol": "jmz",
        "x-killustrator": "kil",
        "x-latex": "latex",
        "x-lyx": "lyx",
        "x-lzx": "lzx",
        "x-maker": [ "frm", "fb", "fbdoc" ],
        "x-ms-wmd": "wmd",
        "x-msdos-program": [ "com", "exe", "bat", "dll" ],
        "x-netcdf": [ "nc" ],
        "x-ns-proxy-autoconfig": [ "pac", "dat" ],
        "x-nwc": "nwc",
        "x-object": "o",
        "x-oz-application": "oza",
        "x-pkcs7-certreqresp": "p7r",
        "x-python-code": [ "pyc", "pyo" ],
        "x-qgis": [ "qgs", "shp", "shx" ],
        "x-quicktimeplayer": "qtl",
        "x-redhat-package-manager": [ "rpm", "rpa" ],
        "x-ruby": "rb",
        "x-sh": "sh",
        "x-shar": "shar",
        "x-shockwave-flash": [ "swf", "swfl" ],
        "x-silverlight": "scr",
        "x-stuffit": "sit",
        "x-sv4cpio": "sv4cpio",
        "x-sv4crc": "sv4crc",
        "x-tar": "tar",
        "x-tex-gf": "gf",
        "x-tex-pk": "pk",
        "x-texinfo": [ "texinfo", "texi" ],
        "x-trash": [ "~", "%", "bak", "old", "sik" ],
        "x-ustar": "ustar",
        "x-wais-source": "src",
        "x-wingz": "wz",
        "x-x509-ca-cert": [ "crt", "der", "cer" ],
        "x-xcf": "xcf",
        "x-xfig": "fig",
        "x-xpinstall": "xpi",
        applixware: "aw",
        "atomsvc+xml": "atomsvc",
        "ccxml+xml": "ccxml",
        "cdmi-capability": "cdmia",
        "cdmi-container": "cdmic",
        "cdmi-domain": "cdmid",
        "cdmi-object": "cdmio",
        "cdmi-queue": "cdmiq",
        "docbook+xml": "dbk",
        "dssc+der": "dssc",
        "dssc+xml": "xdssc",
        "emma+xml": "emma",
        "epub+zip": "epub",
        exi: "exi",
        "font-tdpfr": "pfr",
        "gml+xml": "gml",
        "gpx+xml": "gpx",
        gxf: "gxf",
        hyperstudio: "stk",
        "inkml+xml": [ "ink", "inkml" ],
        ipfix: "ipfix",
        "jsonml+json": "jsonml",
        "lost+xml": "lostxml",
        "mads+xml": "mads",
        marc: "mrc",
        "marcxml+xml": "mrcx",
        "mathml+xml": [ "mathml", "mml" ],
        mbox: "mbox",
        "mediaservercontrol+xml": "mscml",
        "metalink+xml": "metalink",
        "metalink4+xml": "meta4",
        "mets+xml": "mets",
        "mods+xml": "mods",
        mp21: [ "m21", "mp21" ],
        mp4: "mp4s",
        "oebps-package+xml": "opf",
        "omdoc+xml": "omdoc",
        onenote: [ "onetoc", "onetoc2", "onetmp", "onepkg" ],
        oxps: "oxps",
        "patch-ops-error+xml": "xer",
        "pgp-encrypted": "pgp",
        pkcs10: "p10",
        "pkcs7-mime": [ "p7m", "p7c" ],
        "pkcs7-signature": "p7s",
        pkcs8: "p8",
        "pkix-attr-cert": "ac",
        "pkix-crl": "crl",
        "pkix-pkipath": "pkipath",
        pkixcmp: "pki",
        "pls+xml": "pls",
        "prs.cww": "cww",
        "pskc+xml": "pskcxml",
        "reginfo+xml": "rif",
        "relax-ng-compact-syntax": "rnc",
        "resource-lists+xml": "rl",
        "resource-lists-diff+xml": "rld",
        "rls-services+xml": "rs",
        "rpki-ghostbusters": "gbr",
        "rpki-manifest": "mft",
        "rpki-roa": "roa",
        "rsd+xml": "rsd",
        "sbml+xml": "sbml",
        "scvp-cv-request": "scq",
        "scvp-cv-response": "scs",
        "scvp-vp-request": "spq",
        "scvp-vp-response": "spp",
        sdp: "sdp",
        "set-payment-initiation": "setpay",
        "set-registration-initiation": "setreg",
        "shf+xml": "shf",
        "sparql-query": "rq",
        "sparql-results+xml": "srx",
        srgs: "gram",
        "srgs+xml": "grxml",
        "sru+xml": "sru",
        "ssdl+xml": "ssdl",
        "ssml+xml": "ssml",
        "tei+xml": [ "tei", "teicorpus" ],
        "thraud+xml": "tfi",
        "timestamped-data": "tsd",
        "vnd.3gpp.pic-bw-large": "plb",
        "vnd.3gpp.pic-bw-small": "psb",
        "vnd.3gpp.pic-bw-var": "pvb",
        "vnd.3gpp2.tcap": "tcap",
        "vnd.3m.post-it-notes": "pwn",
        "vnd.accpac.simply.aso": "aso",
        "vnd.accpac.simply.imp": "imp",
        "vnd.acucobol": "acu",
        "vnd.acucorp": [ "atc", "acutc" ],
        "vnd.adobe.air-application-installer-package+zip": "air",
        "vnd.adobe.formscentral.fcdt": "fcdt",
        "vnd.adobe.fxp": [ "fxp", "fxpl" ],
        "vnd.adobe.xdp+xml": "xdp",
        "vnd.adobe.xfdf": "xfdf",
        "vnd.ahead.space": "ahead",
        "vnd.airzip.filesecure.azf": "azf",
        "vnd.airzip.filesecure.azs": "azs",
        "vnd.amazon.ebook": "azw",
        "vnd.americandynamics.acc": "acc",
        "vnd.amiga.ami": "ami",
        "vnd.anser-web-certificate-issue-initiation": "cii",
        "vnd.anser-web-funds-transfer-initiation": "fti",
        "vnd.antix.game-component": "atx",
        "vnd.apple.installer+xml": "mpkg",
        "vnd.apple.mpegurl": "m3u8",
        "vnd.aristanetworks.swi": "swi",
        "vnd.astraea-software.iota": "iota",
        "vnd.audiograph": "aep",
        "vnd.blueice.multipass": "mpm",
        "vnd.bmi": "bmi",
        "vnd.businessobjects": "rep",
        "vnd.chemdraw+xml": "cdxml",
        "vnd.chipnuts.karaoke-mmd": "mmd",
        "vnd.claymore": "cla",
        "vnd.cloanto.rp9": "rp9",
        "vnd.clonk.c4group": [ "c4g", "c4d", "c4f", "c4p", "c4u" ],
        "vnd.cluetrust.cartomobile-config": "c11amc",
        "vnd.cluetrust.cartomobile-config-pkg": "c11amz",
        "vnd.commonspace": "csp",
        "vnd.contact.cmsg": "cdbcmsg",
        "vnd.cosmocaller": "cmc",
        "vnd.crick.clicker": "clkx",
        "vnd.crick.clicker.keyboard": "clkk",
        "vnd.crick.clicker.palette": "clkp",
        "vnd.crick.clicker.template": "clkt",
        "vnd.crick.clicker.wordbank": "clkw",
        "vnd.criticaltools.wbs+xml": "wbs",
        "vnd.ctc-posml": "pml",
        "vnd.cups-ppd": "ppd",
        "vnd.curl.car": "car",
        "vnd.curl.pcurl": "pcurl",
        "vnd.dart": "dart",
        "vnd.data-vision.rdz": "rdz",
        "vnd.dece.data": [ "uvf", "uvvf", "uvd", "uvvd" ],
        "vnd.dece.ttml+xml": [ "uvt", "uvvt" ],
        "vnd.dece.unspecified": [ "uvx", "uvvx" ],
        "vnd.dece.zip": [ "uvz", "uvvz" ],
        "vnd.denovo.fcselayout-link": "fe_launch",
        "vnd.dna": "dna",
        "vnd.dolby.mlp": "mlp",
        "vnd.dpgraph": "dpg",
        "vnd.dreamfactory": "dfac",
        "vnd.ds-keypoint": "kpxx",
        "vnd.dvb.ait": "ait",
        "vnd.dvb.service": "svc",
        "vnd.dynageo": "geo",
        "vnd.ecowin.chart": "mag",
        "vnd.enliven": "nml",
        "vnd.epson.esf": "esf",
        "vnd.epson.msf": "msf",
        "vnd.epson.quickanime": "qam",
        "vnd.epson.salt": "slt",
        "vnd.epson.ssf": "ssf",
        "vnd.eszigno3+xml": [ "es3", "et3" ],
        "vnd.ezpix-album": "ez2",
        "vnd.ezpix-package": "ez3",
        "vnd.fdf": "fdf",
        "vnd.fdsn.mseed": "mseed",
        "vnd.fdsn.seed": [ "seed", "dataless" ],
        "vnd.flographit": "gph",
        "vnd.fluxtime.clip": "ftc",
        "vnd.framemaker": [ "fm", "frame", "maker", "book" ],
        "vnd.frogans.fnc": "fnc",
        "vnd.frogans.ltf": "ltf",
        "vnd.fsc.weblaunch": "fsc",
        "vnd.fujitsu.oasys": "oas",
        "vnd.fujitsu.oasys2": "oa2",
        "vnd.fujitsu.oasys3": "oa3",
        "vnd.fujitsu.oasysgp": "fg5",
        "vnd.fujitsu.oasysprs": "bh2",
        "vnd.fujixerox.ddd": "ddd",
        "vnd.fujixerox.docuworks": "xdw",
        "vnd.fujixerox.docuworks.binder": "xbd",
        "vnd.fuzzysheet": "fzs",
        "vnd.genomatix.tuxedo": "txd",
        "vnd.geogebra.file": "ggb",
        "vnd.geogebra.tool": "ggt",
        "vnd.geometry-explorer": [ "gex", "gre" ],
        "vnd.geonext": "gxt",
        "vnd.geoplan": "g2w",
        "vnd.geospace": "g3w",
        "vnd.gmx": "gmx",
        "vnd.grafeq": [ "gqf", "gqs" ],
        "vnd.groove-account": "gac",
        "vnd.groove-help": "ghf",
        "vnd.groove-identity-message": "gim",
        "vnd.groove-injector": "grv",
        "vnd.groove-tool-message": "gtm",
        "vnd.groove-tool-template": "tpl",
        "vnd.groove-vcard": "vcg",
        "vnd.hal+xml": "hal",
        "vnd.handheld-entertainment+xml": "zmm",
        "vnd.hbci": "hbci",
        "vnd.hhe.lesson-player": "les",
        "vnd.hp-hpgl": "hpgl",
        "vnd.hp-hpid": "hpid",
        "vnd.hp-hps": "hps",
        "vnd.hp-jlyt": "jlt",
        "vnd.hp-pcl": "pcl",
        "vnd.hp-pclxl": "pclxl",
        "vnd.hydrostatix.sof-data": "sfd-hdstx",
        "vnd.ibm.minipay": "mpy",
        "vnd.ibm.modcap": [ "afp", "listafp", "list3820" ],
        "vnd.ibm.rights-management": "irm",
        "vnd.ibm.secure-container": "sc",
        "vnd.iccprofile": [ "icc", "icm" ],
        "vnd.igloader": "igl",
        "vnd.immervision-ivp": "ivp",
        "vnd.immervision-ivu": "ivu",
        "vnd.insors.igm": "igm",
        "vnd.intercon.formnet": [ "xpw", "xpx" ],
        "vnd.intergeo": "i2g",
        "vnd.intu.qbo": "qbo",
        "vnd.intu.qfx": "qfx",
        "vnd.ipunplugged.rcprofile": "rcprofile",
        "vnd.irepository.package+xml": "irp",
        "vnd.is-xpr": "xpr",
        "vnd.isac.fcs": "fcs",
        "vnd.jam": "jam",
        "vnd.jcp.javame.midlet-rms": "rms",
        "vnd.jisp": "jisp",
        "vnd.joost.joda-archive": "joda",
        "vnd.kahootz": [ "ktz", "ktr" ],
        "vnd.kde.karbon": "karbon",
        "vnd.kde.kchart": "chrt",
        "vnd.kde.kformula": "kfo",
        "vnd.kde.kivio": "flw",
        "vnd.kde.kontour": "kon",
        "vnd.kde.kpresenter": [ "kpr", "kpt" ],
        "vnd.kde.kspread": "ksp",
        "vnd.kde.kword": [ "kwd", "kwt" ],
        "vnd.kenameaapp": "htke",
        "vnd.kidspiration": "kia",
        "vnd.kinar": [ "kne", "knp" ],
        "vnd.koan": [ "skp", "skd", "skt", "skm" ],
        "vnd.kodak-descriptor": "sse",
        "vnd.las.las+xml": "lasxml",
        "vnd.llamagraphics.life-balance.desktop": "lbd",
        "vnd.llamagraphics.life-balance.exchange+xml": "lbe",
        "vnd.lotus-1-2-3": "123",
        "vnd.lotus-approach": "apr",
        "vnd.lotus-freelance": "pre",
        "vnd.lotus-notes": "nsf",
        "vnd.lotus-organizer": "org",
        "vnd.lotus-screencam": "scm",
        "vnd.lotus-wordpro": "lwp",
        "vnd.macports.portpkg": "portpkg",
        "vnd.mcd": "mcd",
        "vnd.medcalcdata": "mc1",
        "vnd.mediastation.cdkey": "cdkey",
        "vnd.mfer": "mwf",
        "vnd.mfmp": "mfm",
        "vnd.micrografx.flo": "flo",
        "vnd.micrografx.igx": "igx",
        "vnd.mif": "mif",
        "vnd.mobius.daf": "daf",
        "vnd.mobius.dis": "dis",
        "vnd.mobius.mbk": "mbk",
        "vnd.mobius.mqy": "mqy",
        "vnd.mobius.msl": "msl",
        "vnd.mobius.plc": "plc",
        "vnd.mobius.txf": "txf",
        "vnd.mophun.application": "mpn",
        "vnd.mophun.certificate": "mpc",
        "vnd.ms-artgalry": "cil",
        "vnd.ms-cab-compressed": "cab",
        "vnd.ms-excel.addin.macroenabled.12": "xlam",
        "vnd.ms-excel.sheet.binary.macroenabled.12": "xlsb",
        "vnd.ms-excel.sheet.macroenabled.12": "xlsm",
        "vnd.ms-excel.template.macroenabled.12": "xltm",
        "vnd.ms-fontobject": "eot",
        "vnd.ms-htmlhelp": "chm",
        "vnd.ms-ims": "ims",
        "vnd.ms-lrm": "lrm",
        "vnd.ms-officetheme": "thmx",
        "vnd.ms-powerpoint.addin.macroenabled.12": "ppam",
        "vnd.ms-powerpoint.presentation.macroenabled.12": "pptm",
        "vnd.ms-powerpoint.slide.macroenabled.12": "sldm",
        "vnd.ms-powerpoint.slideshow.macroenabled.12": "ppsm",
        "vnd.ms-powerpoint.template.macroenabled.12": "potm",
        "vnd.ms-project": [ "mpp", "mpt" ],
        "vnd.ms-word.document.macroenabled.12": "docm",
        "vnd.ms-word.template.macroenabled.12": "dotm",
        "vnd.ms-works": [ "wps", "wks", "wcm", "wdb" ],
        "vnd.ms-wpl": "wpl",
        "vnd.ms-xpsdocument": "xps",
        "vnd.mseq": "mseq",
        "vnd.musician": "mus",
        "vnd.muvee.style": "msty",
        "vnd.mynfc": "taglet",
        "vnd.neurolanguage.nlu": "nlu",
        "vnd.nitf": [ "ntf", "nitf" ],
        "vnd.noblenet-directory": "nnd",
        "vnd.noblenet-sealer": "nns",
        "vnd.noblenet-web": "nnw",
        "vnd.nokia.n-gage.data": "ngdat",
        "vnd.nokia.n-gage.symbian.install": "n-gage",
        "vnd.nokia.radio-preset": "rpst",
        "vnd.nokia.radio-presets": "rpss",
        "vnd.novadigm.edm": "edm",
        "vnd.novadigm.edx": "edx",
        "vnd.novadigm.ext": "ext",
        "vnd.oasis.opendocument.chart-template": "otc",
        "vnd.oasis.opendocument.formula-template": "odft",
        "vnd.oasis.opendocument.image-template": "oti",
        "vnd.olpc-sugar": "xo",
        "vnd.oma.dd2+xml": "dd2",
        "vnd.openofficeorg.extension": "oxt",
        "vnd.openxmlformats-officedocument.presentationml.slide": "sldx",
        "vnd.osgeo.mapguide.package": "mgp",
        "vnd.osgi.dp": "dp",
        "vnd.osgi.subsystem": "esa",
        "vnd.palm": [ "pdb", "pqa", "oprc" ],
        "vnd.pawaafile": "paw",
        "vnd.pg.format": "str",
        "vnd.pg.osasli": "ei6",
        "vnd.picsel": "efif",
        "vnd.pmi.widget": "wg",
        "vnd.pocketlearn": "plf",
        "vnd.powerbuilder6": "pbd",
        "vnd.previewsystems.box": "box",
        "vnd.proteus.magazine": "mgz",
        "vnd.publishare-delta-tree": "qps",
        "vnd.pvi.ptid1": "ptid",
        "vnd.quark.quarkxpress": [ "qxd", "qxt", "qwd", "qwt", "qxl", "qxb" ],
        "vnd.realvnc.bed": "bed",
        "vnd.recordare.musicxml": "mxl",
        "vnd.recordare.musicxml+xml": "musicxml",
        "vnd.rig.cryptonote": "cryptonote",
        "vnd.rn-realmedia": "rm",
        "vnd.rn-realmedia-vbr": "rmvb",
        "vnd.route66.link66+xml": "link66",
        "vnd.sailingtracker.track": "st",
        "vnd.seemail": "see",
        "vnd.sema": "sema",
        "vnd.semd": "semd",
        "vnd.semf": "semf",
        "vnd.shana.informed.formdata": "ifm",
        "vnd.shana.informed.formtemplate": "itp",
        "vnd.shana.informed.interchange": "iif",
        "vnd.shana.informed.package": "ipk",
        "vnd.simtech-mindmapper": [ "twd", "twds" ],
        "vnd.smart.teacher": "teacher",
        "vnd.solent.sdkm+xml": [ "sdkm", "sdkd" ],
        "vnd.spotfire.dxp": "dxp",
        "vnd.spotfire.sfs": "sfs",
        "vnd.stepmania.package": "smzip",
        "vnd.stepmania.stepchart": "sm",
        "vnd.sus-calendar": [ "sus", "susp" ],
        "vnd.svd": "svd",
        "vnd.syncml+xml": "xsm",
        "vnd.syncml.dm+wbxml": "bdm",
        "vnd.syncml.dm+xml": "xdm",
        "vnd.tao.intent-module-archive": "tao",
        "vnd.tcpdump.pcap": [ "pcap", "cap", "dmp" ],
        "vnd.tmobile-livetv": "tmo",
        "vnd.trid.tpt": "tpt",
        "vnd.triscape.mxs": "mxs",
        "vnd.trueapp": "tra",
        "vnd.ufdl": [ "ufd", "ufdl" ],
        "vnd.uiq.theme": "utz",
        "vnd.umajin": "umj",
        "vnd.unity": "unityweb",
        "vnd.uoml+xml": "uoml",
        "vnd.vcx": "vcx",
        "vnd.visionary": "vis",
        "vnd.vsf": "vsf",
        "vnd.webturbo": "wtb",
        "vnd.wolfram.player": "nbp",
        "vnd.wqd": "wqd",
        "vnd.wt.stf": "stf",
        "vnd.xara": "xar",
        "vnd.xfdl": "xfdl",
        "vnd.yamaha.hv-dic": "hvd",
        "vnd.yamaha.hv-script": "hvs",
        "vnd.yamaha.hv-voice": "hvp",
        "vnd.yamaha.openscoreformat": "osf",
        "vnd.yamaha.openscoreformat.osfpvg+xml": "osfpvg",
        "vnd.yamaha.smaf-audio": "saf",
        "vnd.yamaha.smaf-phrase": "spf",
        "vnd.yellowriver-custom-menu": "cmp",
        "vnd.zul": [ "zir", "zirz" ],
        "vnd.zzazz.deck+xml": "zaz",
        "voicexml+xml": "vxml",
        widget: "wgt",
        winhlp: "hlp",
        "wsdl+xml": "wsdl",
        "wspolicy+xml": "wspolicy",
        "x-ace-compressed": "ace",
        "x-authorware-bin": [ "aab", "x32", "u32", "vox" ],
        "x-authorware-map": "aam",
        "x-authorware-seg": "aas",
        "x-blorb": [ "blb", "blorb" ],
        "x-bzip": "bz",
        "x-bzip2": [ "bz2", "boz" ],
        "x-cfs-compressed": "cfs",
        "x-chat": "chat",
        "x-conference": "nsc",
        "x-dgc-compressed": "dgc",
        "x-dtbncx+xml": "ncx",
        "x-dtbook+xml": "dtb",
        "x-dtbresource+xml": "res",
        "x-eva": "eva",
        "x-font-bdf": "bdf",
        "x-font-ghostscript": "gsf",
        "x-font-linux-psf": "psf",
        "x-font-pcf": "pcf",
        "x-font-snf": "snf",
        "x-font-ttf": [ "ttf", "ttc" ],
        "x-font-type1": [ "pfa", "pfb", "pfm", "afm" ],
        "x-freearc": "arc",
        "x-gca-compressed": "gca",
        "x-glulx": "ulx",
        "x-gramps-xml": "gramps",
        "x-install-instructions": "install",
        "x-lzh-compressed": [ "lzh", "lha" ],
        "x-mie": "mie",
        "x-mobipocket-ebook": [ "prc", "mobi" ],
        "x-ms-application": "application",
        "x-ms-shortcut": "lnk",
        "x-ms-xbap": "xbap",
        "x-msbinder": "obd",
        "x-mscardfile": "crd",
        "x-msclip": "clp",
        "application/x-ms-installer": "msi",
        "x-msmediaview": [ "mvb", "m13", "m14" ],
        "x-msmetafile": [ "wmf", "wmz", "emf", "emz" ],
        "x-msmoney": "mny",
        "x-mspublisher": "pub",
        "x-msschedule": "scd",
        "x-msterminal": "trm",
        "x-mswrite": "wri",
        "x-nzb": "nzb",
        "x-pkcs12": [ "p12", "pfx" ],
        "x-pkcs7-certificates": [ "p7b", "spc" ],
        "x-research-info-systems": "ris",
        "x-silverlight-app": "xap",
        "x-sql": "sql",
        "x-stuffitx": "sitx",
        "x-subrip": "srt",
        "x-t3vm-image": "t3",
        "x-tex-tfm": "tfm",
        "x-tgif": "obj",
        "x-xliff+xml": "xlf",
        "x-xz": "xz",
        "x-zmachine": [ "z1", "z2", "z3", "z4", "z5", "z6", "z7", "z8" ],
        "xaml+xml": "xaml",
        "xcap-diff+xml": "xdf",
        "xenc+xml": "xenc",
        "xml-dtd": "dtd",
        "xop+xml": "xop",
        "xproc+xml": "xpl",
        "xslt+xml": "xslt",
        "xv+xml": [ "mxml", "xhvml", "xvml", "xvm" ],
        yang: "yang",
        "yin+xml": "yin",
        envoy: "evy",
        fractals: "fif",
        "internet-property-stream": "acx",
        olescript: "axs",
        "vnd.ms-outlook": "msg",
        "vnd.ms-pkicertstore": "sst",
        "x-compress": "z",
        "x-perfmon": [ "pma", "pmc", "pmr", "pmw" ],
        "ynd.ms-pkipko": "pko",
        gzip: [ "gz", "tgz" ],
        "smil+xml": [ "smi", "smil" ],
        "vnd.debian.binary-package": [ "deb", "udeb" ],
        "vnd.hzn-3d-crossword": "x3d",
        "vnd.sqlite3": [ "db", "sqlite", "sqlite3", "db-wal", "sqlite-wal", "db-shm", "sqlite-shm" ],
        "vnd.wap.sic": "sic",
        "vnd.wap.slc": "slc",
        "x-krita": [ "kra", "krz" ],
        "x-perl": [ "pm", "pl" ],
        yaml: [ "yaml", "yml" ]
    },
    audio: {
        amr: "amr",
        "amr-wb": "awb",
        annodex: "axa",
        basic: [ "au", "snd" ],
        flac: "flac",
        midi: [ "mid", "midi", "kar", "rmi" ],
        mpeg: [ "mpga", "mpega", "mp3", "m4a", "mp2a", "m2a", "m3a" ],
        mpegurl: "m3u",
        ogg: [ "oga", "ogg", "spx" ],
        "prs.sid": "sid",
        "x-aiff": "aifc",
        "x-gsm": "gsm",
        "x-ms-wma": "wma",
        "x-ms-wax": "wax",
        "x-pn-realaudio": "ram",
        "x-realaudio": "ra",
        "x-sd2": "sd2",
        adpcm: "adp",
        mp4: "mp4a",
        s3m: "s3m",
        silk: "sil",
        "vnd.dece.audio": [ "uva", "uvva" ],
        "vnd.digital-winds": "eol",
        "vnd.dra": "dra",
        "vnd.dts": "dts",
        "vnd.dts.hd": "dtshd",
        "vnd.lucent.voice": "lvp",
        "vnd.ms-playready.media.pya": "pya",
        "vnd.nuera.ecelp4800": "ecelp4800",
        "vnd.nuera.ecelp7470": "ecelp7470",
        "vnd.nuera.ecelp9600": "ecelp9600",
        "vnd.rip": "rip",
        webm: "weba",
        "x-caf": "caf",
        "x-matroska": "mka",
        "x-pn-realaudio-plugin": "rmp",
        xm: "xm",
        aac: "aac",
        aiff: [ "aiff", "aif", "aff" ],
        opus: "opus",
        wav: "wav"
    },
    chemical: {
        "x-alchemy": "alc",
        "x-cache": [ "cac", "cache" ],
        "x-cache-csf": "csf",
        "x-cactvs-binary": [ "cbin", "cascii", "ctab" ],
        "x-cdx": "cdx",
        "x-chem3d": "c3d",
        "x-cif": "cif",
        "x-cmdf": "cmdf",
        "x-cml": "cml",
        "x-compass": "cpa",
        "x-crossfire": "bsd",
        "x-csml": [ "csml", "csm" ],
        "x-ctx": "ctx",
        "x-cxf": [ "cxf", "cef" ],
        "x-embl-dl-nucleotide": [ "emb", "embl" ],
        "x-gamess-input": [ "inp", "gam", "gamin" ],
        "x-gaussian-checkpoint": [ "fch", "fchk" ],
        "x-gaussian-cube": "cub",
        "x-gaussian-input": [ "gau", "gjc", "gjf" ],
        "x-gaussian-log": "gal",
        "x-gcg8-sequence": "gcg",
        "x-genbank": "gen",
        "x-hin": "hin",
        "x-isostar": [ "istr", "ist" ],
        "x-jcamp-dx": [ "jdx", "dx" ],
        "x-kinemage": "kin",
        "x-macmolecule": "mcm",
        "x-macromodel-input": "mmod",
        "x-mdl-molfile": "mol",
        "x-mdl-rdfile": "rd",
        "x-mdl-rxnfile": "rxn",
        "x-mdl-sdfile": "sd",
        "x-mdl-tgf": "tgf",
        "x-mmcif": "mcif",
        "x-mol2": "mol2",
        "x-molconn-Z": "b",
        "x-mopac-graph": "gpt",
        "x-mopac-input": [ "mop", "mopcrt", "zmt" ],
        "x-mopac-out": "moo",
        "x-ncbi-asn1": "asn",
        "x-ncbi-asn1-ascii": [ "prt", "ent" ],
        "x-ncbi-asn1-binary": "val",
        "x-rosdal": "ros",
        "x-swissprot": "sw",
        "x-vamas-iso14976": "vms",
        "x-vmd": "vmd",
        "x-xtel": "xtel",
        "x-xyz": "xyz"
    },
    font: {
        otf: "otf",
        woff: "woff",
        woff2: "woff2"
    },
    image: {
        gif: "gif",
        ief: "ief",
        jpeg: [ "jpeg", "jpg", "jpe", "jfif", "jfif-tbnl", "jif" ],
        pcx: "pcx",
        png: "png",
        "svg+xml": [ "svg", "svgz" ],
        tiff: [ "tiff", "tif" ],
        "vnd.djvu": [ "djvu", "djv" ],
        "vnd.wap.wbmp": "wbmp",
        "x-canon-cr2": "cr2",
        "x-canon-crw": "crw",
        "x-cmu-raster": "ras",
        "x-coreldraw": "cdr",
        "x-coreldrawpattern": "pat",
        "x-coreldrawtemplate": "cdt",
        "x-corelphotopaint": "cpt",
        "x-epson-erf": "erf",
        "x-icon": "ico",
        "x-jg": "art",
        "x-jng": "jng",
        "x-nikon-nef": "nef",
        "x-olympus-orf": "orf",
        "x-portable-anymap": "pnm",
        "x-portable-bitmap": "pbm",
        "x-portable-graymap": "pgm",
        "x-portable-pixmap": "ppm",
        "x-rgb": "rgb",
        "x-xbitmap": "xbm",
        "x-xpixmap": "xpm",
        "x-xwindowdump": "xwd",
        bmp: "bmp",
        cgm: "cgm",
        g3fax: "g3",
        ktx: "ktx",
        "prs.btif": "btif",
        sgi: "sgi",
        "vnd.dece.graphic": [ "uvi", "uvvi", "uvg", "uvvg" ],
        "vnd.dwg": "dwg",
        "vnd.dxf": "dxf",
        "vnd.fastbidsheet": "fbs",
        "vnd.fpx": "fpx",
        "vnd.fst": "fst",
        "vnd.fujixerox.edmics-mmr": "mmr",
        "vnd.fujixerox.edmics-rlc": "rlc",
        "vnd.ms-modi": "mdi",
        "vnd.ms-photo": "wdp",
        "vnd.net-fpx": "npx",
        "vnd.xiff": "xif",
        webp: "webp",
        "x-3ds": "3ds",
        "x-cmx": "cmx",
        "x-freehand": [ "fh", "fhc", "fh4", "fh5", "fh7" ],
        "x-pict": [ "pic", "pct" ],
        "x-tga": "tga",
        "cis-cod": "cod",
        avif: "avifs",
        heic: [ "heif", "heic" ],
        pjpeg: [ "pjpg" ],
        "vnd.adobe.photoshop": "psd",
        "x-adobe-dng": "dng",
        "x-fuji-raf": "raf",
        "x-icns": "icns",
        "x-kodak-dcr": "dcr",
        "x-kodak-k25": "k25",
        "x-kodak-kdc": "kdc",
        "x-minolta-mrw": "mrw",
        "x-panasonic-raw": [ "raw", "rw2", "rwl" ],
        "x-pentax-pef": [ "pef", "ptx" ],
        "x-sigma-x3f": "x3f",
        "x-sony-arw": "arw",
        "x-sony-sr2": "sr2",
        "x-sony-srf": "srf"
    },
    message: {
        rfc822: [ "eml", "mime", "mht", "mhtml", "nws" ]
    },
    model: {
        iges: [ "igs", "iges" ],
        mesh: [ "msh", "mesh", "silo" ],
        vrml: [ "wrl", "vrml" ],
        "x3d+vrml": [ "x3dv", "x3dvz" ],
        "x3d+xml": "x3dz",
        "x3d+binary": [ "x3db", "x3dbz" ],
        "vnd.collada+xml": "dae",
        "vnd.dwf": "dwf",
        "vnd.gdl": "gdl",
        "vnd.gtw": "gtw",
        "vnd.mts": "mts",
        "vnd.usdz+zip": "usdz",
        "vnd.vtu": "vtu"
    },
    text: {
        "cache-manifest": [ "manifest", "appcache" ],
        calendar: [ "ics", "icz", "ifb" ],
        css: "css",
        csv: "csv",
        h323: "323",
        html: [ "html", "htm", "shtml", "stm" ],
        iuls: "uls",
        plain: [ "txt", "text", "brf", "conf", "def", "list", "log", "in", "bas", "diff", "ksh" ],
        richtext: "rtx",
        scriptlet: [ "sct", "wsc" ],
        texmacs: "tm",
        "tab-separated-values": "tsv",
        "vnd.sun.j2me.app-descriptor": "jad",
        "vnd.wap.wml": "wml",
        "vnd.wap.wmlscript": "wmls",
        "x-bibtex": "bib",
        "x-boo": "boo",
        "x-c++hdr": [ "h++", "hpp", "hxx", "hh" ],
        "x-c++src": [ "c++", "cpp", "cxx", "cc" ],
        "x-component": "htc",
        "x-dsrc": "d",
        "x-diff": "patch",
        "x-haskell": "hs",
        "x-java": "java",
        "x-literate-haskell": "lhs",
        "x-moc": "moc",
        "x-pascal": [ "p", "pas", "pp", "inc" ],
        "x-pcs-gcd": "gcd",
        "x-python": "py",
        "x-scala": "scala",
        "x-setext": "etx",
        "x-tcl": [ "tcl", "tk" ],
        "x-tex": [ "tex", "ltx", "sty", "cls" ],
        "x-vcalendar": "vcs",
        "x-vcard": "vcf",
        n3: "n3",
        "prs.lines.tag": "dsc",
        sgml: [ "sgml", "sgm" ],
        troff: [ "t", "tr", "roff", "man", "me", "ms" ],
        turtle: "ttl",
        "uri-list": [ "uri", "uris", "urls" ],
        vcard: "vcard",
        "vnd.curl": "curl",
        "vnd.curl.dcurl": "dcurl",
        "vnd.curl.scurl": "scurl",
        "vnd.curl.mcurl": "mcurl",
        "vnd.dvb.subtitle": "sub",
        "vnd.fly": "fly",
        "vnd.fmi.flexstor": "flx",
        "vnd.graphviz": "gv",
        "vnd.in3d.3dml": "3dml",
        "vnd.in3d.spot": "spot",
        "x-asm": [ "s", "asm" ],
        "x-c": [ "c", "h", "dic" ],
        "x-fortran": [ "f", "for", "f77", "f90" ],
        "x-opml": "opml",
        "x-nfo": "nfo",
        "x-sfv": "sfv",
        "x-uuencode": "uu",
        webviewhtml: "htt",
        javascript: "js",
        json: "json",
        markdown: [ "md", "markdown", "mdown", "markdn" ],
        "vnd.wap.si": "si",
        "vnd.wap.sl": "sl"
    },
    video: {
        avif: "avif",
        "3gpp": "3gp",
        annodex: "axv",
        dl: "dl",
        dv: [ "dif", "dv" ],
        fli: "fli",
        gl: "gl",
        mpeg: [ "mpeg", "mpg", "mpe", "m1v", "m2v", "mp2", "mpa", "mpv2" ],
        mp4: [ "mp4", "mp4v", "mpg4" ],
        quicktime: [ "qt", "mov" ],
        ogg: "ogv",
        "vnd.mpegurl": [ "mxu", "m4u" ],
        "x-flv": "flv",
        "x-la-asf": [ "lsf", "lsx" ],
        "x-mng": "mng",
        "x-ms-asf": [ "asf", "asx", "asr" ],
        "x-ms-wm": "wm",
        "x-ms-wmv": "wmv",
        "x-ms-wmx": "wmx",
        "x-ms-wvx": "wvx",
        "x-msvideo": "avi",
        "x-sgi-movie": "movie",
        "x-matroska": [ "mpv", "mkv", "mk3d", "mks" ],
        "3gpp2": "3g2",
        h261: "h261",
        h263: "h263",
        h264: "h264",
        jpeg: "jpgv",
        jpm: [ "jpm", "jpgm" ],
        mj2: [ "mj2", "mjp2" ],
        "vnd.dece.hd": [ "uvh", "uvvh" ],
        "vnd.dece.mobile": [ "uvm", "uvvm" ],
        "vnd.dece.pd": [ "uvp", "uvvp" ],
        "vnd.dece.sd": [ "uvs", "uvvs" ],
        "vnd.dece.video": [ "uvv", "uvvv" ],
        "vnd.dvb.file": "dvb",
        "vnd.fvt": "fvt",
        "vnd.ms-playready.media.pyv": "pyv",
        "vnd.uvvu.mp4": [ "uvu", "uvvu" ],
        "vnd.vivo": "viv",
        webm: "webm",
        "x-f4v": "f4v",
        "x-m4v": "m4v",
        "x-ms-vob": "vob",
        "x-smv": "smv",
        mp2t: "ts"
    },
    "x-conference": {
        "x-cooltalk": "ice"
    },
    "x-world": {
        "x-vrml": [ "vrm", "flr", "wrz", "xaf", "xof" ]
    }
}, $e = (() => {
    const e = {};
    for (const t of Object.keys(He)) {
        for (const r of Object.keys(He[t])) {
            const n = He[t][r];
            if ("string" == typeof n) {
                e[n] = t + "/" + r;
            } else {
                for (let o = 0; o < n.length; o++) {
                    e[n[o]] = t + "/" + r;
                }
            }
        }
    }
    return e;
})();

const Ge = [];

for (let e = 0; e < 256; e++) {
    let t = e;
    for (let e = 0; e < 8; e++) {
        1 & t ? t = t >>> 1 ^ 3988292384 : t >>>= 1;
    }
    Ge[e] = t;
}

class We {
    constructor(e) {
        this.crc = e || -1;
    }
    append(e) {
        let t = 0 | this.crc;
        for (let r = 0, n = 0 | e.length; r < n; r++) {
            t = t >>> 8 ^ Ge[255 & (t ^ e[r])];
        }
        this.crc = t;
    }
    get() {
        return ~this.crc;
    }
}

class Ve extends TransformStream {
    constructor() {
        let e;
        const t = new We;
        super({
            transform(e, r) {
                t.append(e), r.enqueue(e);
            },
            flush() {
                const r = new Uint8Array(4);
                new DataView(r.buffer).setUint32(0, t.get()), e.value = r;
            }
        }), e = this;
    }
}

function Ke(e) {
    if (typeof TextEncoder == Ie) {
        e = unescape(encodeURIComponent(e));
        const t = new Uint8Array(e.length);
        for (let r = 0; r < t.length; r++) {
            t[r] = e.charCodeAt(r);
        }
        return t;
    }
    return (new TextEncoder).encode(e);
}

const qe = {
    concat(e, t) {
        if (0 === e.length || 0 === t.length) {
            return e.concat(t);
        }
        const r = e[e.length - 1], n = qe.getPartial(r);
        return 32 === n ? e.concat(t) : qe._shiftRight(t, n, 0 | r, e.slice(0, e.length - 1));
    },
    bitLength(e) {
        const t = e.length;
        if (0 === t) {
            return 0;
        }
        const r = e[t - 1];
        return 32 * (t - 1) + qe.getPartial(r);
    },
    clamp(e, t) {
        if (32 * e.length < t) {
            return e;
        }
        const r = (e = e.slice(0, Math.ceil(t / 32))).length;
        return t &= 31, r > 0 && t && (e[r - 1] = qe.partial(t, e[r - 1] & 2147483648 >> t - 1, 1)), 
        e;
    },
    partial: (e, t, r) => 32 === e ? t : (r ? 0 | t : t << 32 - e) + 1099511627776 * e,
    getPartial: e => Math.round(e / 1099511627776) || 32,
    _shiftRight(e, t, r, n) {
        for (void 0 === n && (n = []); t >= 32; t -= 32) {
            n.push(r), r = 0;
        }
        if (0 === t) {
            return n.concat(e);
        }
        for (let o = 0; o < e.length; o++) {
            n.push(r | e[o] >>> t), r = e[o] << 32 - t;
        }
        const o = e.length ? e[e.length - 1] : 0, i = qe.getPartial(o);
        return n.push(qe.partial(t + i & 31, t + i > 32 ? r : n.pop(), 1)), n;
    }
}, Je = {
    bytes: {
        fromBits(e) {
            const t = qe.bitLength(e) / 8, r = new Uint8Array(t);
            let n;
            for (let o = 0; o < t; o++) {
                3 & o || (n = e[o / 4]), r[o] = n >>> 24, n <<= 8;
            }
            return r;
        },
        toBits(e) {
            const t = [];
            let r, n = 0;
            for (r = 0; r < e.length; r++) {
                n = n << 8 | e[r], 3 & ~r || (t.push(n), n = 0);
            }
            return 3 & r && t.push(qe.partial(8 * (3 & r), n)), t;
        }
    }
}, Xe = {
    sha1: class {
        constructor(e) {
            const t = this;
            t.blockSize = 512, t._init = [ 1732584193, 4023233417, 2562383102, 271733878, 3285377520 ], 
            t._key = [ 1518500249, 1859775393, 2400959708, 3395469782 ], e ? (t._h = e._h.slice(0), 
            t._buffer = e._buffer.slice(0), t._length = e._length) : t.reset();
        }
        reset() {
            const e = this;
            return e._h = e._init.slice(0), e._buffer = [], e._length = 0, e;
        }
        update(e) {
            const t = this;
            "string" == typeof e && (e = Je.utf8String.toBits(e));
            const r = t._buffer = qe.concat(t._buffer, e), n = t._length, o = t._length = n + qe.bitLength(e);
            if (o > 9007199254740991) {
                throw new Error("Cannot hash more than 2^53 - 1 bits");
            }
            const i = new Uint32Array(r);
            let a = 0;
            for (let e = t.blockSize + n - (t.blockSize + n & t.blockSize - 1); e <= o; e += t.blockSize) {
                t._block(i.subarray(16 * a, 16 * (a + 1))), a += 1;
            }
            return r.splice(0, 16 * a), t;
        }
        finalize() {
            const e = this;
            let t = e._buffer;
            const r = e._h;
            t = qe.concat(t, [ qe.partial(1, 1) ]);
            for (let e = t.length + 2; 15 & e; e++) {
                t.push(0);
            }
            for (t.push(Math.floor(e._length / 4294967296)), t.push(0 | e._length); t.length; ) {
                e._block(t.splice(0, 16));
            }
            return e.reset(), r;
        }
        _f(e, t, r, n) {
            return e <= 19 ? t & r | ~t & n : e <= 39 ? t ^ r ^ n : e <= 59 ? t & r | t & n | r & n : e <= 79 ? t ^ r ^ n : void 0;
        }
        _S(e, t) {
            return t << e | t >>> 32 - e;
        }
        _block(e) {
            const t = this, r = t._h, n = Array(80);
            for (let t = 0; t < 16; t++) {
                n[t] = e[t];
            }
            let o = r[0], i = r[1], a = r[2], s = r[3], u = r[4];
            for (let e = 0; e <= 79; e++) {
                e >= 16 && (n[e] = t._S(1, n[e - 3] ^ n[e - 8] ^ n[e - 14] ^ n[e - 16]));
                const r = t._S(5, o) + t._f(e, i, a, s) + u + n[e] + t._key[Math.floor(e / 20)] | 0;
                u = s, s = a, a = t._S(30, i), i = o, o = r;
            }
            r[0] = r[0] + o | 0, r[1] = r[1] + i | 0, r[2] = r[2] + a | 0, r[3] = r[3] + s | 0, 
            r[4] = r[4] + u | 0;
        }
    }
}, Ze = {
    aes: class {
        constructor(e) {
            const t = this;
            t._tables = [ [ [], [], [], [], [] ], [ [], [], [], [], [] ] ], t._tables[0][0][0] || t._precompute();
            const r = t._tables[0][4], n = t._tables[1], o = e.length;
            let i, a, s, u = 1;
            if (4 !== o && 6 !== o && 8 !== o) {
                throw new Error("invalid aes key size");
            }
            for (t._key = [ a = e.slice(0), s = [] ], i = o; i < 4 * o + 28; i++) {
                let e = a[i - 1];
                (i % o === 0 || 8 === o && i % o === 4) && (e = r[e >>> 24] << 24 ^ r[e >> 16 & 255] << 16 ^ r[e >> 8 & 255] << 8 ^ r[255 & e], 
                i % o === 0 && (e = e << 8 ^ e >>> 24 ^ u << 24, u = u << 1 ^ 283 * (u >> 7))), 
                a[i] = a[i - o] ^ e;
            }
            for (let e = 0; i; e++, i--) {
                const t = a[3 & e ? i : i - 4];
                s[e] = i <= 4 || e < 4 ? t : n[0][r[t >>> 24]] ^ n[1][r[t >> 16 & 255]] ^ n[2][r[t >> 8 & 255]] ^ n[3][r[255 & t]];
            }
        }
        encrypt(e) {
            return this._crypt(e, 0);
        }
        decrypt(e) {
            return this._crypt(e, 1);
        }
        _precompute() {
            const e = this._tables[0], t = this._tables[1], r = e[4], n = t[4], o = [], i = [];
            let a, s, u, l;
            for (let e = 0; e < 256; e++) {
                i[(o[e] = e << 1 ^ 283 * (e >> 7)) ^ e] = e;
            }
            for (let c = a = 0; !r[c]; c ^= s || 1, a = i[a] || 1) {
                let i = a ^ a << 1 ^ a << 2 ^ a << 3 ^ a << 4;
                i = i >> 8 ^ 255 & i ^ 99, r[c] = i, n[i] = c, l = o[u = o[s = o[c]]];
                let f = 16843009 * l ^ 65537 * u ^ 257 * s ^ 16843008 * c, d = 257 * o[i] ^ 16843008 * i;
                for (let r = 0; r < 4; r++) {
                    e[r][c] = d = d << 24 ^ d >>> 8, t[r][i] = f = f << 24 ^ f >>> 8;
                }
            }
            for (let r = 0; r < 5; r++) {
                e[r] = e[r].slice(0), t[r] = t[r].slice(0);
            }
        }
        _crypt(e, t) {
            if (4 !== e.length) {
                throw new Error("invalid aes block size");
            }
            const r = this._key[t], n = r.length / 4 - 2, o = [ 0, 0, 0, 0 ], i = this._tables[t], a = i[0], s = i[1], u = i[2], l = i[3], c = i[4];
            let f, d, p, h = e[0] ^ r[0], v = e[t ? 3 : 1] ^ r[1], g = e[2] ^ r[2], m = e[t ? 1 : 3] ^ r[3], y = 4;
            for (let e = 0; e < n; e++) {
                f = a[h >>> 24] ^ s[v >> 16 & 255] ^ u[g >> 8 & 255] ^ l[255 & m] ^ r[y], d = a[v >>> 24] ^ s[g >> 16 & 255] ^ u[m >> 8 & 255] ^ l[255 & h] ^ r[y + 1], 
                p = a[g >>> 24] ^ s[m >> 16 & 255] ^ u[h >> 8 & 255] ^ l[255 & v] ^ r[y + 2], m = a[m >>> 24] ^ s[h >> 16 & 255] ^ u[v >> 8 & 255] ^ l[255 & g] ^ r[y + 3], 
                y += 4, h = f, v = d, g = p;
            }
            for (let e = 0; e < 4; e++) {
                o[t ? 3 & -e : e] = c[h >>> 24] << 24 ^ c[v >> 16 & 255] << 16 ^ c[g >> 8 & 255] << 8 ^ c[255 & m] ^ r[y++], 
                f = h, h = v, v = g, g = m, m = f;
            }
            return o;
        }
    }
}, Ye = {
    getRandomValues(e) {
        const t = new Uint32Array(e.buffer), r = e => {
            let t = 987654321;
            const r = 4294967295;
            return function() {
                t = 36969 * (65535 & t) + (t >> 16) & r;
                return (((t << 16) + (e = 18e3 * (65535 & e) + (e >> 16) & r) & r) / 4294967296 + .5) * (Math.random() > .5 ? 1 : -1);
            };
        };
        for (let n, o = 0; o < e.length; o += 4) {
            const e = r(4294967296 * (n || Math.random()));
            n = 987654071 * e(), t[o / 4] = 4294967296 * e() | 0;
        }
        return e;
    }
}, Qe = {
    ctrGladman: class {
        constructor(e, t) {
            this._prf = e, this._initIv = t, this._iv = t;
        }
        reset() {
            this._iv = this._initIv;
        }
        update(e) {
            return this.calculate(this._prf, e, this._iv);
        }
        incWord(e) {
            if (255 & ~(e >> 24)) {
                e += 1 << 24;
            } else {
                let t = e >> 16 & 255, r = e >> 8 & 255, n = 255 & e;
                255 === t ? (t = 0, 255 === r ? (r = 0, 255 === n ? n = 0 : ++n) : ++r) : ++t, e = 0, 
                e += t << 16, e += r << 8, e += n;
            }
            return e;
        }
        incCounter(e) {
            0 === (e[0] = this.incWord(e[0])) && (e[1] = this.incWord(e[1]));
        }
        calculate(e, t, r) {
            let n;
            if (!(n = t.length)) {
                return [];
            }
            const o = qe.bitLength(t);
            for (let o = 0; o < n; o += 4) {
                this.incCounter(r);
                const n = e.encrypt(r);
                t[o] ^= n[0], t[o + 1] ^= n[1], t[o + 2] ^= n[2], t[o + 3] ^= n[3];
            }
            return qe.clamp(t, o);
        }
    }
}, et = {
    importKey: e => new et.hmacSha1(Je.bytes.toBits(e)),
    pbkdf2(e, t, r, n) {
        if (r = r || 1e4, n < 0 || r < 0) {
            throw new Error("invalid params to pbkdf2");
        }
        const o = 1 + (n >> 5) << 2;
        let i, a, s, u, l;
        const c = new ArrayBuffer(o), f = new DataView(c);
        let d = 0;
        const p = qe;
        for (t = Je.bytes.toBits(t), l = 1; d < (o || 1); l++) {
            for (i = a = e.encrypt(p.concat(t, [ l ])), s = 1; s < r; s++) {
                for (a = e.encrypt(a), u = 0; u < a.length; u++) {
                    i[u] ^= a[u];
                }
            }
            for (s = 0; d < (o || 1) && s < i.length; s++) {
                f.setInt32(d, i[s]), d += 4;
            }
        }
        return c.slice(0, n / 8);
    },
    hmacSha1: class {
        constructor(e) {
            const t = this, r = t._hash = Xe.sha1, n = [ [], [] ];
            t._baseHash = [ new r, new r ];
            const o = t._baseHash[0].blockSize / 32;
            e.length > o && (e = (new r).update(e).finalize());
            for (let t = 0; t < o; t++) {
                n[0][t] = 909522486 ^ e[t], n[1][t] = 1549556828 ^ e[t];
            }
            t._baseHash[0].update(n[0]), t._baseHash[1].update(n[1]), t._resultHash = new r(t._baseHash[0]);
        }
        reset() {
            const e = this;
            e._resultHash = new e._hash(e._baseHash[0]), e._updated = !1;
        }
        update(e) {
            this._updated = !0, this._resultHash.update(e);
        }
        digest() {
            const e = this, t = e._resultHash.finalize(), r = new e._hash(e._baseHash[1]).update(t).finalize();
            return e.reset(), r;
        }
        encrypt(e) {
            if (this._updated) {
                throw new Error("encrypt on already updated hmac called!");
            }
            return this.update(e), this.digest(e);
        }
    }
}, tt = typeof crypto != Ie && typeof crypto.getRandomValues == ke, rt = "Invalid password", nt = "Invalid signature", ot = "zipjs-abort-check-password";

function it(e) {
    return tt ? crypto.getRandomValues(e) : Ye.getRandomValues(e);
}

const at = 16, st = {
    name: "PBKDF2"
}, ut = Object.assign({
    hash: {
        name: "HMAC"
    }
}, st), lt = Object.assign({
    iterations: 1e3,
    hash: {
        name: "SHA-1"
    }
}, st), ct = [ "deriveBits" ], ft = [ 8, 12, 16 ], dt = [ 16, 24, 32 ], pt = 10, ht = [ 0, 0, 0, 0 ], vt = typeof crypto != Ie, gt = vt && crypto.subtle, mt = vt && typeof gt != Ie, yt = Je.bytes, _t = Ze.aes, Et = Qe.ctrGladman, bt = et.hmacSha1;

let wt = vt && mt && typeof gt.importKey == ke, Dt = vt && mt && typeof gt.deriveBits == ke;

class St extends TransformStream {
    constructor({password: e, rawPassword: t, signed: r, encryptionStrength: n, checkPasswordOnly: o}) {
        super({
            start() {
                Object.assign(this, {
                    ready: new Promise(e => this.resolveReady = e),
                    password: xt(e, t),
                    signed: r,
                    strength: n - 1,
                    pending: new Uint8Array
                });
            },
            async transform(e, t) {
                const r = this, {password: n, strength: i, resolveReady: a, ready: s} = r;
                n ? (await async function(e, t, r, n) {
                    const o = await Ct(e, t, r, Mt(n, 0, ft[t])), i = Mt(n, ft[t]);
                    if (o[0] != i[0] || o[1] != i[1]) {
                        throw new Error(rt);
                    }
                }(r, i, n, Mt(e, 0, ft[i] + 2)), e = Mt(e, ft[i] + 2), o ? t.error(new Error(ot)) : a()) : await s;
                const u = new Uint8Array(e.length - pt - (e.length - pt) % at);
                t.enqueue(Ot(r, e, u, 0, pt, !0));
            },
            async flush(e) {
                const {signed: t, ctr: r, hmac: n, pending: o, ready: i} = this;
                if (n && r) {
                    await i;
                    const a = Mt(o, 0, o.length - pt), s = Mt(o, o.length - pt);
                    let u = new Uint8Array;
                    if (a.length) {
                        const e = It(yt, a);
                        n.update(e);
                        const t = r.update(e);
                        u = Pt(yt, t);
                    }
                    if (t) {
                        const e = Mt(Pt(yt, n.digest()), 0, pt);
                        for (let t = 0; t < pt; t++) {
                            if (e[t] != s[t]) {
                                throw new Error(nt);
                            }
                        }
                    }
                    e.enqueue(u);
                }
            }
        });
    }
}

class At extends TransformStream {
    constructor({password: e, rawPassword: t, encryptionStrength: r}) {
        let n;
        super({
            start() {
                Object.assign(this, {
                    ready: new Promise(e => this.resolveReady = e),
                    password: xt(e, t),
                    strength: r - 1,
                    pending: new Uint8Array
                });
            },
            async transform(e, t) {
                const r = this, {password: n, strength: o, resolveReady: i, ready: a} = r;
                let s = new Uint8Array;
                n ? (s = await async function(e, t, r) {
                    const n = it(new Uint8Array(ft[t])), o = await Ct(e, t, r, n);
                    return Ft(n, o);
                }(r, o, n), i()) : await a;
                const u = new Uint8Array(s.length + e.length - e.length % at);
                u.set(s, 0), t.enqueue(Ot(r, e, u, s.length, 0));
            },
            async flush(e) {
                const {ctr: t, hmac: r, pending: o, ready: i} = this;
                if (r && t) {
                    await i;
                    let a = new Uint8Array;
                    if (o.length) {
                        const e = t.update(It(yt, o));
                        r.update(e), a = Pt(yt, e);
                    }
                    n.signature = Pt(yt, r.digest()).slice(0, pt), e.enqueue(Ft(a, n.signature));
                }
            }
        }), n = this;
    }
}

function Ot(e, t, r, n, o, i) {
    const {ctr: a, hmac: s, pending: u} = e, l = t.length - o;
    let c;
    for (u.length && (t = Ft(u, t), r = function(e, t) {
        if (t && t > e.length) {
            const r = e;
            (e = new Uint8Array(t)).set(r, 0);
        }
        return e;
    }(r, l - l % at)), c = 0; c <= l - at; c += at) {
        const e = It(yt, Mt(t, c, c + at));
        i && s.update(e);
        const o = a.update(e);
        i || s.update(o), r.set(Pt(yt, o), c + n);
    }
    return e.pending = Mt(t, c), r;
}

async function Ct(e, t, r, n) {
    e.password = null;
    const o = await async function(e, t, r, n, o) {
        if (!wt) {
            return et.importKey(t);
        }
        try {
            return await gt.importKey(e, t, r, n, o);
        } catch (e) {
            return wt = !1, et.importKey(t);
        }
    }("raw", r, ut, !1, ct), i = await async function(e, t, r) {
        if (!Dt) {
            return et.pbkdf2(t, e.salt, lt.iterations, r);
        }
        try {
            return await gt.deriveBits(e, t, r);
        } catch (n) {
            return Dt = !1, et.pbkdf2(t, e.salt, lt.iterations, r);
        }
    }(Object.assign({
        salt: n
    }, lt), o, 8 * (2 * dt[t] + 2)), a = new Uint8Array(i), s = It(yt, Mt(a, 0, dt[t])), u = It(yt, Mt(a, dt[t], 2 * dt[t])), l = Mt(a, 2 * dt[t]);
    return Object.assign(e, {
        keys: {
            key: s,
            authentication: u,
            passwordVerification: l
        },
        ctr: new Et(new _t(s), Array.from(ht)),
        hmac: new bt(u)
    }), l;
}

function xt(e, t) {
    return t === Pe ? Ke(e) : t;
}

function Ft(e, t) {
    let r = e;
    return e.length + t.length && (r = new Uint8Array(e.length + t.length), r.set(e, 0), 
    r.set(t, e.length)), r;
}

function Mt(e, t, r) {
    return e.subarray(t, r);
}

function Pt(e, t) {
    return e.fromBits(t);
}

function It(e, t) {
    return e.toBits(t);
}

const kt = 12;

class Rt extends TransformStream {
    constructor({password: e, passwordVerification: t, checkPasswordOnly: r}) {
        super({
            start() {
                Object.assign(this, {
                    password: e,
                    passwordVerification: t
                }), Nt(this, e);
            },
            transform(e, t) {
                const n = this;
                if (n.password) {
                    const t = jt(n, e.subarray(0, kt));
                    if (n.password = null, t[11] != n.passwordVerification) {
                        throw new Error(rt);
                    }
                    e = e.subarray(kt);
                }
                r ? t.error(new Error(ot)) : t.enqueue(jt(n, e));
            }
        });
    }
}

class Tt extends TransformStream {
    constructor({password: e, passwordVerification: t}) {
        super({
            start() {
                Object.assign(this, {
                    password: e,
                    passwordVerification: t
                }), Nt(this, e);
            },
            transform(e, t) {
                const r = this;
                let n, o;
                if (r.password) {
                    r.password = null;
                    const t = it(new Uint8Array(kt));
                    t[11] = r.passwordVerification, n = new Uint8Array(e.length + t.length), n.set(Lt(r, t), 0), 
                    o = kt;
                } else {
                    n = new Uint8Array(e.length), o = 0;
                }
                n.set(Lt(r, e), o), t.enqueue(n);
            }
        });
    }
}

function jt(e, t) {
    const r = new Uint8Array(t.length);
    for (let n = 0; n < t.length; n++) {
        r[n] = Ut(e) ^ t[n], Bt(e, r[n]);
    }
    return r;
}

function Lt(e, t) {
    const r = new Uint8Array(t.length);
    for (let n = 0; n < t.length; n++) {
        r[n] = Ut(e) ^ t[n], Bt(e, t[n]);
    }
    return r;
}

function Nt(e, t) {
    const r = [ 305419896, 591751049, 878082192 ];
    Object.assign(e, {
        keys: r,
        crcKey0: new We(r[0]),
        crcKey2: new We(r[2])
    });
    for (let r = 0; r < t.length; r++) {
        Bt(e, t.charCodeAt(r));
    }
}

function Bt(e, t) {
    let [r, n, o] = e.keys;
    e.crcKey0.append([ t ]), r = ~e.crcKey0.get(), n = Ht(Math.imul(Ht(n + zt(r)), 134775813) + 1), 
    e.crcKey2.append([ n >>> 24 ]), o = ~e.crcKey2.get(), e.keys = [ r, n, o ];
}

function Ut(e) {
    const t = 2 | e.keys[2];
    return zt(Math.imul(t, 1 ^ t) >>> 8);
}

function zt(e) {
    return 255 & e;
}

function Ht(e) {
    return 4294967295 & e;
}

const $t = "deflate-raw";

class Gt extends TransformStream {
    constructor(e, {chunkSize: t, CompressionStream: r, CompressionStreamNative: n}) {
        super({});
        const {compressed: o, encrypted: i, useCompressionStream: a, zipCrypto: s, signed: u, level: l} = e, c = this;
        let f, d, p = Vt(super.readable);
        i && !s || !u || (f = new Ve, p = Jt(p, f)), o && (p = qt(p, a, {
            level: l,
            chunkSize: t
        }, n, r)), i && (s ? p = Jt(p, new Tt(e)) : (d = new At(e), p = Jt(p, d))), Kt(c, p, () => {
            let e;
            i && !s && (e = d.signature), i && !s || !u || (e = new DataView(f.value.buffer).getUint32(0)), 
            c.signature = e;
        });
    }
}

class Wt extends TransformStream {
    constructor(e, {chunkSize: t, DecompressionStream: r, DecompressionStreamNative: n}) {
        super({});
        const {zipCrypto: o, encrypted: i, signed: a, signature: s, compressed: u, useCompressionStream: l} = e;
        let c, f, d = Vt(super.readable);
        i && (o ? d = Jt(d, new Rt(e)) : (f = new St(e), d = Jt(d, f))), u && (d = qt(d, l, {
            chunkSize: t
        }, n, r)), i && !o || !a || (c = new Ve, d = Jt(d, c)), Kt(this, d, () => {
            if ((!i || o) && a) {
                const e = new DataView(c.value.buffer);
                if (s != e.getUint32(0, !1)) {
                    throw new Error(nt);
                }
            }
        });
    }
}

function Vt(e) {
    return Jt(e, new TransformStream({
        transform(e, t) {
            e && e.length && t.enqueue(e);
        }
    }));
}

function Kt(e, t, r) {
    t = Jt(t, new TransformStream({
        flush: r
    })), Object.defineProperty(e, "readable", {
        get: () => t
    });
}

function qt(e, t, r, n, o) {
    try {
        e = Jt(e, new (t && n ? n : o)($t, r));
    } catch (n) {
        if (!t) {
            return e;
        }
        try {
            e = Jt(e, new o($t, r));
        } catch (t) {
            return e;
        }
    }
    return e;
}

function Jt(e, t) {
    return e.pipeThrough(t);
}

const Xt = "message", Zt = "start", Yt = "pull", Qt = "data", er = "close", tr = "deflate", rr = "inflate";

class nr extends TransformStream {
    constructor(e, t) {
        super({});
        const r = this, {codecType: n} = e;
        let o;
        n.startsWith(tr) ? o = Gt : n.startsWith(rr) && (o = Wt);
        let i = 0, a = 0;
        const s = new o(e, t), u = super.readable, l = new TransformStream({
            transform(e, t) {
                e && e.length && (a += e.length, t.enqueue(e));
            },
            flush() {
                Object.assign(r, {
                    inputSize: a
                });
            }
        }), c = new TransformStream({
            transform(e, t) {
                e && e.length && (i += e.length, t.enqueue(e));
            },
            flush() {
                const {signature: e} = s;
                Object.assign(r, {
                    signature: e,
                    outputSize: i,
                    inputSize: a
                });
            }
        });
        Object.defineProperty(r, "readable", {
            get: () => u.pipeThrough(l).pipeThrough(s).pipeThrough(c)
        });
    }
}

class or extends TransformStream {
    constructor(e) {
        let t;
        super({
            transform: function r(n, o) {
                if (t) {
                    const e = new Uint8Array(t.length + n.length);
                    e.set(t), e.set(n, t.length), n = e, t = null;
                }
                n.length > e ? (o.enqueue(n.slice(0, e)), r(n.slice(e), o)) : t = n;
            },
            flush(e) {
                t && t.length && e.enqueue(t);
            }
        });
    }
}

let ir = typeof Worker != Ie;

class ar {
    constructor(e, {readable: t, writable: r}, {options: n, config: o, streamOptions: i, useWebWorkers: a, transferStreams: s, scripts: u}, l) {
        const {signal: c} = i;
        return Object.assign(e, {
            busy: !0,
            readable: t.pipeThrough(new or(o.chunkSize)).pipeThrough(new sr(t, i), {
                signal: c
            }),
            writable: r,
            options: Object.assign({}, n),
            scripts: u,
            transferStreams: s,
            terminate: () => new Promise(t => {
                const {worker: r, busy: n} = e;
                r ? (n ? e.resolveTerminated = t : (r.terminate(), t()), e.interface = null) : t();
            }),
            onTaskFinished() {
                const {resolveTerminated: t} = e;
                t && (e.resolveTerminated = null, e.terminated = !0, e.worker.terminate(), t()), 
                e.busy = !1, l(e);
            }
        }), (a && ir ? cr : lr)(e, o);
    }
}

class sr extends TransformStream {
    constructor(e, {onstart: t, onprogress: r, size: n, onend: o}) {
        let i = 0;
        super({
            async start() {
                t && await ur(t, n);
            },
            async transform(e, t) {
                i += e.length, r && await ur(r, i, n), t.enqueue(e);
            },
            async flush() {
                e.size = i, o && await ur(o, i);
            }
        });
    }
}

async function ur(e, ...t) {
    try {
        await e(...t);
    } catch (e) {}
}

function lr(e, t) {
    return {
        run: () => async function({options: e, readable: t, writable: r, onTaskFinished: n}, o) {
            try {
                const n = new nr(e, o);
                await t.pipeThrough(n).pipeTo(r, {
                    preventClose: !0,
                    preventAbort: !0
                });
                const {signature: i, inputSize: a, outputSize: s} = n;
                return {
                    signature: i,
                    inputSize: a,
                    outputSize: s
                };
            } finally {
                n();
            }
        }(e, t)
    };
}

function cr(e, t) {
    const {baseURL: r, chunkSize: n} = t;
    if (!e.interface) {
        let o;
        try {
            o = function(e, t, r) {
                const n = {
                    type: "module"
                };
                let o, i;
                typeof e == ke && (e = e());
                try {
                    o = new URL(e, t);
                } catch (t) {
                    o = e;
                }
                if (fr) {
                    try {
                        i = new Worker(o);
                    } catch (e) {
                        fr = !1, i = new Worker(o, n);
                    }
                } else {
                    i = new Worker(o, n);
                }
                return i.addEventListener(Xt, e => async function({data: e}, t) {
                    const {type: r, value: n, messageId: o, result: i, error: a} = e, {reader: s, writer: u, resolveResult: l, rejectResult: c, onTaskFinished: f} = t;
                    try {
                        if (a) {
                            const {message: e, stack: t, code: r, name: n} = a, o = new Error(e);
                            Object.assign(o, {
                                stack: t,
                                code: r,
                                name: n
                            }), d(o);
                        } else {
                            if (r == Yt) {
                                const {value: e, done: r} = await s.read();
                                pr({
                                    type: Qt,
                                    value: e,
                                    done: r,
                                    messageId: o
                                }, t);
                            }
                            r == Qt && (await u.ready, await u.write(new Uint8Array(n)), pr({
                                type: "ack",
                                messageId: o
                            }, t)), r == er && d(null, i);
                        }
                    } catch (a) {
                        pr({
                            type: er,
                            messageId: o
                        }, t), d(a);
                    }
                    function d(e, t) {
                        e ? c(e) : l(t), u && u.releaseLock(), f();
                    }
                }(e, r)), i;
            }(e.scripts[0], r, e);
        } catch (r) {
            return ir = !1, lr(e, t);
        }
        Object.assign(e, {
            worker: o,
            interface: {
                run: () => async function(e, t) {
                    let r, n;
                    const o = new Promise((e, t) => {
                        r = e, n = t;
                    });
                    Object.assign(e, {
                        reader: null,
                        writer: null,
                        resolveResult: r,
                        rejectResult: n,
                        result: o
                    });
                    const {readable: i, options: a, scripts: s} = e, {writable: u, closed: l} = function(e) {
                        let t;
                        const r = new Promise(e => t = e), n = new WritableStream({
                            async write(t) {
                                const r = e.getWriter();
                                await r.ready, await r.write(t), r.releaseLock();
                            },
                            close() {
                                t();
                            },
                            abort: t => e.getWriter().abort(t)
                        });
                        return {
                            writable: n,
                            closed: r
                        };
                    }(e.writable), c = pr({
                        type: Zt,
                        scripts: s.slice(1),
                        options: a,
                        config: t,
                        readable: i,
                        writable: u
                    }, e);
                    c || Object.assign(e, {
                        reader: i.getReader(),
                        writer: u.getWriter()
                    });
                    const f = await o;
                    c || await u.getWriter().close();
                    return await l, f;
                }(e, {
                    chunkSize: n
                })
            }
        });
    }
    return e.interface;
}

let fr = !0, dr = !0;

function pr(e, {worker: t, writer: r, onTaskFinished: n, transferStreams: o}) {
    try {
        let {value: r, readable: n, writable: i} = e;
        const a = [];
        if (r && (r.byteLength < r.buffer.byteLength ? e.value = r.buffer.slice(0, r.byteLength) : e.value = r.buffer, 
        a.push(e.value)), o && dr ? (n && a.push(n), i && a.push(i)) : e.readable = e.writable = null, 
        a.length) {
            try {
                return t.postMessage(e, a), !0;
            } catch (r) {
                dr = !1, e.readable = e.writable = null, t.postMessage(e);
            }
        } else {
            t.postMessage(e);
        }
    } catch (e) {
        throw r && r.releaseLock(), n(), e;
    }
}

let hr = [];

const vr = [];

let gr = 0;

async function mr(e, t) {
    const {options: r, config: n} = t, {transferStreams: o, useWebWorkers: i, useCompressionStream: a, codecType: s, compressed: u, signed: l, encrypted: c} = r, {workerScripts: f, maxWorkers: d} = n;
    t.transferStreams = o || o === Pe;
    const p = !(u || l || c || t.transferStreams);
    return t.useWebWorkers = !p && (i || i === Pe && n.useWebWorkers), t.scripts = t.useWebWorkers && f ? f[s] : [], 
    r.useCompressionStream = a || a === Pe && n.useCompressionStream, (await async function() {
        const r = hr.find(e => !e.busy);
        if (r) {
            return yr(r), new ar(r, e, t, h);
        }
        if (hr.length < d) {
            const r = {
                indexWorker: gr
            };
            return gr++, hr.push(r), new ar(r, e, t, h);
        }
        return new Promise(r => vr.push({
            resolve: r,
            stream: e,
            workerOptions: t
        }));
    }()).run();
    function h(e) {
        if (vr.length) {
            const [{resolve: t, stream: r, workerOptions: n}] = vr.splice(0, 1);
            t(new ar(e, r, n, h));
        } else {
            e.worker ? (yr(e), function(e, t) {
                const {config: r} = t, {terminateWorkerTimeout: n} = r;
                Number.isFinite(n) && n >= 0 && (e.terminated ? e.terminated = !1 : e.terminateTimeout = setTimeout(async () => {
                    hr = hr.filter(t => t != e);
                    try {
                        await e.terminate();
                    } catch (e) {}
                }, n));
            }(e, t)) : hr = hr.filter(t => t != e);
        }
    }
}

function yr(e) {
    const {terminateTimeout: t} = e;
    t && (clearTimeout(t), e.terminateTimeout = null);
}

function _r(e, t, r) {
    return class {
        constructor(n) {
            const o = this;
            (function(e, t) {
                return typeof Object.hasOwn === ke ? Object.hasOwn(e, t) : e.hasOwnProperty(t);
            })(n, "level") && n.level === Pe && delete n.level, o.codec = new e(Object.assign({}, t, n)), 
            r(o.codec, e => {
                if (o.pendingData) {
                    const t = o.pendingData;
                    o.pendingData = new Uint8Array(t.length + e.length);
                    const {pendingData: r} = o;
                    r.set(t, 0), r.set(e, t.length);
                } else {
                    o.pendingData = new Uint8Array(e);
                }
            });
        }
        append(e) {
            return this.codec.push(e), n(this);
        }
        flush() {
            return this.codec.push(new Uint8Array, !0), n(this);
        }
    };
    function n(e) {
        if (e.pendingData) {
            const t = e.pendingData;
            return e.pendingData = null, t;
        }
        return new Uint8Array;
    }
}

const Er = "HTTP error ", br = "HTTP Range not supported", wr = "Writer iterator completed too soon", Dr = "Range", Sr = "GET", Ar = "bytes", Or = 65536, Cr = "writable";

let xr = class {
    constructor() {
        this.size = 0;
    }
    init() {
        this.initialized = !0;
    }
};

class Fr extends xr {
    get readable() {
        const e = this, {chunkSize: t = Or} = e, r = new ReadableStream({
            start() {
                this.chunkOffset = 0;
            },
            async pull(n) {
                const {offset: o = 0, size: i, diskNumberStart: a} = r, {chunkOffset: s} = this;
                n.enqueue(await on(e, o + s, Math.min(t, i - s), a)), s + t > i ? n.close() : this.chunkOffset += t;
            }
        });
        return r;
    }
}

class Mr extends xr {
    constructor() {
        super();
        const e = this, t = new WritableStream({
            write: t => e.writeUint8Array(t)
        });
        Object.defineProperty(e, Cr, {
            get: () => t
        });
    }
    writeUint8Array() {}
}

class Pr extends Fr {
    constructor(e) {
        super();
        let t = e.length;
        for (;"=" == e.charAt(t - 1); ) {
            t--;
        }
        const r = e.indexOf(",") + 1;
        Object.assign(this, {
            dataURI: e,
            dataStart: r,
            size: Math.floor(.75 * (t - r))
        });
    }
    readUint8Array(e, t) {
        const {dataStart: r, dataURI: n} = this, o = new Uint8Array(t), i = 4 * Math.floor(e / 3), a = atob(n.substring(i + r, 4 * Math.ceil((e + t) / 3) + r)), s = e - 3 * Math.floor(i / 4);
        for (let e = s; e < s + t; e++) {
            o[e - s] = a.charCodeAt(e);
        }
        return o;
    }
}

class Ir extends Mr {
    constructor(e) {
        super(), Object.assign(this, {
            data: "data:" + (e || "") + ";base64,",
            pending: []
        });
    }
    writeUint8Array(e) {
        const t = this;
        let r = 0, n = t.pending;
        const o = t.pending.length;
        for (t.pending = "", r = 0; r < 3 * Math.floor((o + e.length) / 3) - o; r++) {
            n += String.fromCharCode(e[r]);
        }
        for (;r < e.length; r++) {
            t.pending += String.fromCharCode(e[r]);
        }
        n.length > 2 ? t.data += btoa(n) : t.pending = n;
    }
    getData() {
        return this.data + btoa(this.pending);
    }
}

class kr extends Fr {
    constructor(e) {
        super(), Object.assign(this, {
            blob: e,
            size: e.size
        });
    }
    async readUint8Array(e, t) {
        const r = this, n = e + t, o = e || n < r.size ? r.blob.slice(e, n) : r.blob;
        let i = await o.arrayBuffer();
        return i.byteLength > t && (i = i.slice(e, n)), new Uint8Array(i);
    }
}

class Rr extends xr {
    constructor(e) {
        super();
        const t = new TransformStream, r = [];
        e && r.push([ "Content-Type", e ]), Object.defineProperty(this, Cr, {
            get: () => t.writable
        }), this.blob = new Response(t.readable, {
            headers: r
        }).blob();
    }
    getData() {
        return this.blob;
    }
}

class Tr extends kr {
    constructor(e) {
        super(new Blob([ e ], {
            type: "text/plain"
        }));
    }
}

class jr extends Rr {
    constructor(e) {
        super(e), Object.assign(this, {
            encoding: e,
            utf8: !e || "utf-8" == e.toLowerCase()
        });
    }
    async getData() {
        const {encoding: e, utf8: t} = this, r = await super.getData();
        if (r.text && t) {
            return r.text();
        }
        {
            const t = new FileReader;
            return new Promise((n, o) => {
                Object.assign(t, {
                    onload: ({target: e}) => n(e.result),
                    onerror: () => o(t.error)
                }), t.readAsText(r, e);
            });
        }
    }
}

class Lr extends Fr {
    constructor(e, t) {
        super(), Br(this, e, t);
    }
    async init() {
        await Ur(this, qr, Gr), super.init();
    }
    readUint8Array(e, t) {
        return zr(this, e, t, qr, Gr);
    }
}

class Nr extends Fr {
    constructor(e, t) {
        super(), Br(this, e, t);
    }
    async init() {
        await Ur(this, Jr, Wr), super.init();
    }
    readUint8Array(e, t) {
        return zr(this, e, t, Jr, Wr);
    }
}

function Br(e, t, r) {
    const {preventHeadRequest: n, useRangeHeader: o, forceRangeRequests: i, combineSizeEocd: a} = r;
    delete (r = Object.assign({}, r)).preventHeadRequest, delete r.useRangeHeader, delete r.forceRangeRequests, 
    delete r.combineSizeEocd, delete r.useXHR, Object.assign(e, {
        url: t,
        options: r,
        preventHeadRequest: n,
        useRangeHeader: o,
        forceRangeRequests: i,
        combineSizeEocd: a
    });
}

async function Ur(e, t, r) {
    const {url: n, preventHeadRequest: o, useRangeHeader: i, forceRangeRequests: a, combineSizeEocd: s} = e;
    if (function(e) {
        const {baseURL: t} = Ne(), {protocol: r} = new URL(e, t);
        return "http:" == r || "https:" == r;
    }(n) && (i || a) && (void 0 === o || o)) {
        const n = await t(Sr, e, Hr(e, s ? -22 : void 0));
        if (!a && n.headers.get("Accept-Ranges") != Ar) {
            throw new Error(br);
        }
        {
            let o;
            s && (e.eocdCache = new Uint8Array(await n.arrayBuffer()));
            const i = n.headers.get("Content-Range");
            if (i) {
                const e = i.trim().split(/\s*\/\s*/);
                if (e.length) {
                    const t = e[1];
                    t && "*" != t && (o = Number(t));
                }
            }
            o === Pe ? await Kr(e, t, r) : e.size = o;
        }
    } else {
        await Kr(e, t, r);
    }
}

async function zr(e, t, r, n, o) {
    const {useRangeHeader: i, forceRangeRequests: a, eocdCache: s, size: u, options: l} = e;
    if (i || a) {
        if (s && t == u - De && r == De) {
            return s;
        }
        const o = await n(Sr, e, Hr(e, t, r));
        if (206 != o.status) {
            throw new Error(br);
        }
        return new Uint8Array(await o.arrayBuffer());
    }
    {
        const {data: n} = e;
        return n || await o(e, l), new Uint8Array(e.data.subarray(t, t + r));
    }
}

function Hr(e, t = 0, r = 1) {
    return Object.assign({}, $r(e), {
        [Dr]: Ar + "=" + (t < 0 ? t : t + "-" + (t + r - 1))
    });
}

function $r({options: e}) {
    const {headers: t} = e;
    if (t) {
        return Symbol.iterator in t ? Object.fromEntries(t) : t;
    }
}

async function Gr(e) {
    await Vr(e, qr);
}

async function Wr(e) {
    await Vr(e, Jr);
}

async function Vr(e, t) {
    const r = await t(Sr, e, $r(e));
    e.data = new Uint8Array(await r.arrayBuffer()), e.size || (e.size = e.data.length);
}

async function Kr(e, t, r) {
    if (e.preventHeadRequest) {
        await r(e, e.options);
    } else {
        const n = (await t("HEAD", e, $r(e))).headers.get("Content-Length");
        n ? e.size = Number(n) : await r(e, e.options);
    }
}

async function qr(e, {options: t, url: r}, n) {
    const o = await fetch(r, Object.assign({}, t, {
        method: e,
        headers: n
    }));
    if (o.status < 400) {
        return o;
    }
    throw 416 == o.status ? new Error(br) : new Error(Er + (o.statusText || o.status));
}

function Jr(e, {url: t}, r) {
    return new Promise((n, o) => {
        const i = new XMLHttpRequest;
        if (i.addEventListener("load", () => {
            if (i.status < 400) {
                const e = [];
                i.getAllResponseHeaders().trim().split(/[\r\n]+/).forEach(t => {
                    const r = t.trim().split(/\s*:\s*/);
                    r[0] = r[0].trim().replace(/^[a-z]|-[a-z]/g, e => e.toUpperCase()), e.push(r);
                }), n({
                    status: i.status,
                    arrayBuffer: () => i.response,
                    headers: new Map(e)
                });
            } else {
                o(416 == i.status ? new Error(br) : new Error(Er + (i.statusText || i.status)));
            }
        }, !1), i.addEventListener("error", e => o(e.detail ? e.detail.error : new Error("Network error")), !1), 
        i.open(e, t), r) {
            for (const e of Object.entries(r)) {
                i.setRequestHeader(e[0], e[1]);
            }
        }
        i.responseType = "arraybuffer", i.send();
    });
}

class Xr extends Fr {
    constructor(e, t = {}) {
        super(), Object.assign(this, {
            url: e,
            reader: t.useXHR ? new Nr(e, t) : new Lr(e, t)
        });
    }
    set size(e) {}
    get size() {
        return this.reader.size;
    }
    async init() {
        await this.reader.init(), super.init();
    }
    readUint8Array(e, t) {
        return this.reader.readUint8Array(e, t);
    }
}

class Zr extends Fr {
    constructor(e) {
        super(), Object.assign(this, {
            array: e,
            size: e.length
        });
    }
    readUint8Array(e, t) {
        return this.array.slice(e, e + t);
    }
}

class Yr extends Mr {
    init(e = 0) {
        Object.assign(this, {
            offset: 0,
            array: new Uint8Array(e)
        }), super.init();
    }
    writeUint8Array(e) {
        const t = this;
        if (t.offset + e.length > t.array.length) {
            const r = t.array;
            t.array = new Uint8Array(r.length + e.length), t.array.set(r);
        }
        t.array.set(e, t.offset), t.offset += e.length;
    }
    getData() {
        return this.array;
    }
}

class Qr extends Fr {
    constructor(e) {
        super(), this.readers = e;
    }
    async init() {
        const e = this, {readers: t} = e;
        e.lastDiskNumber = 0, e.lastDiskOffset = 0, await Promise.all(t.map(async (r, n) => {
            await r.init(), n != t.length - 1 && (e.lastDiskOffset += r.size), e.size += r.size;
        })), super.init();
    }
    async readUint8Array(e, t, r = 0) {
        const n = this, {readers: o} = this;
        let i, a = r;
        -1 == a && (a = o.length - 1);
        let s = e;
        for (;s >= o[a].size; ) {
            s -= o[a].size, a++;
        }
        const u = o[a], l = u.size;
        if (s + t <= l) {
            i = await on(u, s, t);
        } else {
            const o = l - s;
            i = new Uint8Array(t), i.set(await on(u, s, o)), i.set(await n.readUint8Array(e + o, t - o, r), o);
        }
        return n.lastDiskNumber = Math.max(a, n.lastDiskNumber), i;
    }
}

class en extends xr {
    constructor(e, t = 4294967295) {
        super();
        const r = this;
        let n, o, i;
        Object.assign(r, {
            diskNumber: 0,
            diskOffset: 0,
            size: 0,
            maxSize: t,
            availableSize: t
        });
        const a = new WritableStream({
            async write(t) {
                const {availableSize: a} = r;
                if (i) {
                    t.length >= a ? (await s(t.slice(0, a)), await u(), r.diskOffset += n.size, r.diskNumber++, 
                    i = null, await this.write(t.slice(a))) : await s(t);
                } else {
                    const {value: a, done: s} = await e.next();
                    if (s && !a) {
                        throw new Error(wr);
                    }
                    n = a, n.size = 0, n.maxSize && (r.maxSize = n.maxSize), r.availableSize = r.maxSize, 
                    await tn(n), o = a.writable, i = o.getWriter(), await this.write(t);
                }
            },
            async close() {
                await i.ready, await u();
            }
        });
        async function s(e) {
            const t = e.length;
            t && (await i.ready, await i.write(e), n.size += t, r.size += t, r.availableSize -= t);
        }
        async function u() {
            o.size = n.size, await i.close();
        }
        Object.defineProperty(r, Cr, {
            get: () => a
        });
    }
}

async function tn(e, t) {
    if (!e.init || e.initialized) {
        return Promise.resolve();
    }
    await e.init(t);
}

function rn(e) {
    return Array.isArray(e) && (e = new Qr(e)), e instanceof ReadableStream && (e = {
        readable: e
    }), e;
}

function nn(e) {
    e.writable === Pe && typeof e.next == ke && (e = new en(e)), e instanceof WritableStream && (e = {
        writable: e
    });
    const {writable: t} = e;
    return t.size === Pe && (t.size = 0), e instanceof en || Object.assign(e, {
        diskNumber: 0,
        diskOffset: 0,
        availableSize: 1 / 0,
        maxSize: 1 / 0
    }), e;
}

function on(e, t, r, n) {
    return e.readUint8Array(t, r, n);
}

const an = Qr, sn = en, un = "\0☺☻♥♦♣♠•◘○◙♂♀♪♫☼►◄↕‼¶§▬↨↑↓→←∟↔▲▼ !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~⌂ÇüéâäàåçêëèïîìÄÅÉæÆôöòûùÿÖÜ¢£¥₧ƒáíóúñÑªº¿⌐¬½¼¡«»░▒▓│┤╡╢╖╕╣║╗╝╜╛┐└┴┬├─┼╞╟╚╔╩╦╠═╬╧╨╤╥╙╘╒╓╫╪┘┌█▄▌▐▀αßΓπΣσµτΦΘΩδ∞φε∩≡±≥≤⌠⌡÷≈°∙·√ⁿ²■ ".split(""), ln = 256 == un.length;

function cn(e, t) {
    return t && "cp437" == t.trim().toLowerCase() ? function(e) {
        if (ln) {
            let t = "";
            for (let r = 0; r < e.length; r++) {
                t += un[e[r]];
            }
            return t;
        }
        return (new TextDecoder).decode(e);
    }(e) : new TextDecoder(t).decode(e);
}

const fn = "filename", dn = "rawFilename", pn = "comment", hn = "rawComment", vn = "uncompressedSize", gn = "compressedSize", mn = "offset", yn = "diskNumberStart", _n = "lastModDate", En = "rawLastModDate", bn = "lastAccessDate", wn = "rawLastAccessDate", Dn = "creationDate", Sn = "rawCreationDate", An = "internalFileAttribute", On = "externalFileAttribute", Cn = "msDosCompatible", xn = "zip64", Fn = [ fn, dn, gn, vn, _n, En, pn, hn, bn, Dn, mn, yn, yn, An, On, Cn, xn, "directory", "bitFlag", "encrypted", "signature", "filenameUTF8", "commentUTF8", "compressionMethod", "version", "versionMadeBy", "extraField", "rawExtraField", "extraFieldZip64", "extraFieldUnicodePath", "extraFieldUnicodeComment", "extraFieldAES", "extraFieldNTFS", "extraFieldExtendedTimestamp" ];

class Mn {
    constructor(e) {
        Fn.forEach(t => this[t] = e[t]);
    }
}

const Pn = "File format is not recognized", In = "End of central directory not found", kn = "End of Zip64 central directory locator not found", Rn = "Central directory header not found", Tn = "Local file header not found", jn = "Zip64 extra field not found", Ln = "File contains encrypted entry", Nn = "Encryption method not supported", Bn = "Compression method not supported", Un = "Split zip file", zn = "utf-8", Hn = "cp437", $n = [ [ vn, he ], [ gn, he ], [ mn, he ], [ yn, ve ] ], Gn = {
    [ve]: {
        getValue: to,
        bytes: 4
    },
    [he]: {
        getValue: ro,
        bytes: 8
    }
};

class Wn {
    constructor(e, t = {}) {
        Object.assign(this, {
            reader: rn(e),
            options: t,
            config: Ne()
        });
    }
    async* getEntriesGenerator(e = {}) {
        const t = this;
        let {reader: r} = t;
        const {config: n} = t;
        if (await tn(r), r.size !== Pe && r.readUint8Array || (r = new kr(await new Response(r.readable).blob()), 
        await tn(r)), r.size < De) {
            throw new Error(Pn);
        }
        r.chunkSize = Be(n);
        const o = await async function(e, t, r, n, o) {
            const i = new Uint8Array(4);
            !function(e, t, r) {
                e.setUint32(t, r, !0);
            }(no(i), 0, t);
            const a = n + o;
            return await s(n) || await s(Math.min(a, r));
            async function s(t) {
                const o = r - t, a = await on(e, o, t);
                for (let e = a.length - n; e >= 0; e--) {
                    if (a[e] == i[0] && a[e + 1] == i[1] && a[e + 2] == i[2] && a[e + 3] == i[3]) {
                        return {
                            offset: o + e,
                            buffer: a.slice(e, e + n).buffer
                        };
                    }
                }
            }
        }(r, Ee, r.size, De, 1048560);
        if (!o) {
            throw to(no(await on(r, 0, 4))) == me ? new Error(Un) : new Error(In);
        }
        const i = no(o);
        let a = to(i, 12), s = to(i, 16);
        const u = o.offset, l = eo(i, 20), c = u + De + l;
        let f = eo(i, 4);
        const d = r.lastDiskNumber || 0;
        let p = eo(i, 6), h = eo(i, 8), v = 0, g = 0;
        if (s == he || a == he || h == ve || p == ve) {
            const e = no(await on(r, o.offset - 20, 20));
            if (to(e, 0) == we) {
                s = ro(e, 8);
                let t = await on(r, s, 56, -1), n = no(t);
                const i = o.offset - 20 - 56;
                if (to(n, 0) != be && s != i) {
                    const e = s;
                    s = i, v = s - e, t = await on(r, s, 56, -1), n = no(t);
                }
                if (to(n, 0) != be) {
                    throw new Error(kn);
                }
                f == ve && (f = to(n, 16)), p == ve && (p = to(n, 20)), h == ve && (h = ro(n, 32)), 
                a == he && (a = ro(n, 40)), s -= a;
            }
        }
        if (s >= r.size && (v = r.size - s - a - De, s = r.size - a - De), d != f) {
            throw new Error(Un);
        }
        if (s < 0) {
            throw new Error(Pn);
        }
        let m = 0, y = await on(r, s, a, p), _ = no(y);
        if (a) {
            const e = o.offset - a;
            if (to(_, m) != _e && s != e) {
                const t = s;
                s = e, v += s - t, y = await on(r, s, a, p), _ = no(y);
            }
        }
        const E = o.offset - s - (r.lastDiskOffset || 0);
        if (a != E && E >= 0 && (a = E, y = await on(r, s, a, p), _ = no(y)), s < 0 || s >= r.size) {
            throw new Error(Pn);
        }
        const b = Xn(t, e, "filenameEncoding"), w = Xn(t, e, "commentEncoding");
        for (let o = 0; o < h; o++) {
            const i = new Vn(r, n, t.options);
            if (to(_, m) != _e) {
                throw new Error(Rn);
            }
            Kn(i, _, m + 6);
            const a = Boolean(i.bitFlag.languageEncodingFlag), s = m + 46, u = s + i.filenameLength, l = u + i.extraFieldLength, c = eo(_, m + 4), f = !(0 & c), d = y.subarray(s, u), p = eo(_, m + 32), E = l + p, D = y.subarray(l, E), S = a, A = a, O = f && !(16 & ~Qn(_, m + 38)), C = to(_, m + 42) + v;
            Object.assign(i, {
                versionMadeBy: c,
                msDosCompatible: f,
                compressedSize: 0,
                uncompressedSize: 0,
                commentLength: p,
                directory: O,
                offset: C,
                diskNumberStart: eo(_, m + 34),
                internalFileAttribute: eo(_, m + 36),
                externalFileAttribute: to(_, m + 38),
                rawFilename: d,
                filenameUTF8: S,
                commentUTF8: A,
                rawExtraField: y.subarray(u, l)
            });
            const x = Xn(t, e, "decodeText") || cn, F = S ? zn : b || Hn, M = A ? zn : w || Hn;
            let P = x(d, F);
            P === Pe && (P = cn(d, F));
            let I = x(D, M);
            I === Pe && (I = cn(D, M)), Object.assign(i, {
                rawComment: D,
                filename: P,
                comment: I,
                directory: O || P.endsWith(xe)
            }), g = Math.max(C, g), await qn(i, i, _, m + 6);
            const k = new Mn(i);
            k.getData = (e, t) => i.getData(e, k, t), m = E;
            const {onprogress: R} = e;
            if (R) {
                try {
                    await R(o + 1, h, new Mn(i));
                } catch (e) {}
            }
            yield k;
        }
        const D = Xn(t, e, "extractPrependedData"), S = Xn(t, e, "extractAppendedData");
        return D && (t.prependedData = g > 0 ? await on(r, 0, g) : new Uint8Array), t.comment = l ? await on(r, u + De, l) : new Uint8Array, 
        S && (t.appendedData = c < r.size ? await on(r, c, r.size - c) : new Uint8Array), 
        !0;
    }
    async getEntries(e = {}) {
        const t = [];
        for await (const r of this.getEntriesGenerator(e)) {
            t.push(r);
        }
        return t;
    }
    async close() {}
}

let Vn = class {
    constructor(e, t, r) {
        Object.assign(this, {
            reader: e,
            config: t,
            options: r
        });
    }
    async getData(e, t, r = {}) {
        const n = this, {reader: o, offset: i, diskNumberStart: a, extraFieldAES: s, compressionMethod: u, config: l, bitFlag: c, signature: f, rawLastModDate: d, uncompressedSize: p, compressedSize: h} = n, v = t.localDirectory = {}, g = no(await on(o, i, 30, a));
        let m = Xn(n, r, "password"), y = Xn(n, r, "rawPassword");
        if (m = m && m.length && m, y = y && y.length && y, s && 99 != s.originalCompressionMethod) {
            throw new Error(Bn);
        }
        if (0 != u && 8 != u) {
            throw new Error(Bn);
        }
        if (to(g, 0) != ge) {
            throw new Error(Tn);
        }
        Kn(v, g, 4), v.rawExtraField = v.extraFieldLength ? await on(o, i + 30 + v.filenameLength, v.extraFieldLength, a) : new Uint8Array, 
        await qn(n, v, g, 4, !0), Object.assign(t, {
            lastAccessDate: v.lastAccessDate,
            creationDate: v.creationDate
        });
        const _ = n.encrypted && v.encrypted, E = _ && !s;
        if (_) {
            if (!E && s.strength === Pe) {
                throw new Error(Nn);
            }
            if (!m && !y) {
                throw new Error(Ln);
            }
        }
        const b = i + 30 + v.filenameLength + v.extraFieldLength, w = h, D = o.readable;
        Object.assign(D, {
            diskNumberStart: a,
            offset: b,
            size: w
        });
        const S = Xn(n, r, "signal"), A = Xn(n, r, "checkPasswordOnly");
        A && (e = new WritableStream), e = nn(e), await tn(e, p);
        const {writable: O} = e, {onstart: C, onprogress: x, onend: F} = r, M = {
            options: {
                codecType: rr,
                password: m,
                rawPassword: y,
                zipCrypto: E,
                encryptionStrength: s && s.strength,
                signed: Xn(n, r, "checkSignature"),
                passwordVerification: E && (c.dataDescriptor ? d >>> 8 & 255 : f >>> 24 & 255),
                signature: f,
                compressed: 0 != u,
                encrypted: _,
                useWebWorkers: Xn(n, r, "useWebWorkers"),
                useCompressionStream: Xn(n, r, "useCompressionStream"),
                transferStreams: Xn(n, r, "transferStreams"),
                checkPasswordOnly: A
            },
            config: l,
            streamOptions: {
                signal: S,
                size: w,
                onstart: C,
                onprogress: x,
                onend: F
            }
        };
        let P = 0;
        try {
            ({outputSize: P} = await mr({
                readable: D,
                writable: O
            }, M));
        } catch (e) {
            if (!A || e.message != ot) {
                throw e;
            }
        } finally {
            const e = Xn(n, r, "preventClose");
            O.size += P, e || O.locked || await O.getWriter().close();
        }
        return A ? Pe : e.getData ? e.getData() : O;
    }
};

function Kn(e, t, r) {
    const n = e.rawBitFlag = eo(t, r + 2), o = !(1 & ~n), i = to(t, r + 6);
    Object.assign(e, {
        encrypted: o,
        version: eo(t, r),
        bitFlag: {
            level: (6 & n) >> 1,
            dataDescriptor: !(8 & ~n),
            languageEncodingFlag: (n & Ce) == Ce
        },
        rawLastModDate: i,
        lastModDate: Zn(i),
        filenameLength: eo(t, r + 22),
        extraFieldLength: eo(t, r + 24)
    });
}

async function qn(e, t, r, n, o) {
    const {rawExtraField: i} = t, a = t.extraField = new Map, s = no(new Uint8Array(i));
    let u = 0;
    try {
        for (;u < i.length; ) {
            const e = eo(s, u), t = eo(s, u + 2);
            a.set(e, {
                type: e,
                data: i.slice(u + 4, u + 4 + t)
            }), u += 4 + t;
        }
    } catch (e) {}
    const l = eo(r, n + 4);
    Object.assign(t, {
        signature: to(r, n + 10),
        uncompressedSize: to(r, n + 18),
        compressedSize: to(r, n + 14)
    });
    const c = a.get(1);
    c && (!function(e, t) {
        t.zip64 = !0;
        const r = no(e.data), n = $n.filter(([e, r]) => t[e] == r);
        for (let o = 0, i = 0; o < n.length; o++) {
            const [a, s] = n[o];
            if (t[a] == s) {
                const n = Gn[s];
                t[a] = e[a] = n.getValue(r, i), i += n.bytes;
            } else if (e[a]) {
                throw new Error(jn);
            }
        }
    }(c, t), t.extraFieldZip64 = c);
    const f = a.get(28789);
    f && (await Jn(f, fn, dn, t, e), t.extraFieldUnicodePath = f);
    const d = a.get(25461);
    d && (await Jn(d, pn, hn, t, e), t.extraFieldUnicodeComment = d);
    const p = a.get(39169);
    p ? (!function(e, t, r) {
        const n = no(e.data), o = Qn(n, 4);
        Object.assign(e, {
            vendorVersion: Qn(n, 0),
            vendorId: Qn(n, 2),
            strength: o,
            originalCompressionMethod: r,
            compressionMethod: eo(n, 5)
        }), t.compressionMethod = e.compressionMethod;
    }(p, t, l), t.extraFieldAES = p) : t.compressionMethod = l;
    const h = a.get(10);
    h && (!function(e, t) {
        const r = no(e.data);
        let n, o = 4;
        try {
            for (;o < e.data.length && !n; ) {
                const t = eo(r, o), i = eo(r, o + 2);
                1 == t && (n = e.data.slice(o + 4, o + 4 + i)), o += 4 + i;
            }
        } catch (e) {}
        try {
            if (n && 24 == n.length) {
                const r = no(n), o = r.getBigUint64(0, !0), i = r.getBigUint64(8, !0), a = r.getBigUint64(16, !0);
                Object.assign(e, {
                    rawLastModDate: o,
                    rawLastAccessDate: i,
                    rawCreationDate: a
                });
                const s = Yn(o), u = Yn(i), l = {
                    lastModDate: s,
                    lastAccessDate: u,
                    creationDate: Yn(a)
                };
                Object.assign(e, l), Object.assign(t, l);
            }
        } catch (e) {}
    }(h, t), t.extraFieldNTFS = h);
    const v = a.get(Ae);
    v && (!function(e, t, r) {
        const n = no(e.data), o = Qn(n, 0), i = [], a = [];
        r ? (1 & ~o || (i.push(_n), a.push(En)), 2 & ~o || (i.push(bn), a.push(wn)), 4 & ~o || (i.push(Dn), 
        a.push(Sn))) : e.data.length >= 5 && (i.push(_n), a.push(En));
        let s = 1;
        i.forEach((r, o) => {
            if (e.data.length >= s + 4) {
                const i = to(n, s);
                t[r] = e[r] = new Date(1e3 * i);
                const u = a[o];
                e[u] = i;
            }
            s += 4;
        });
    }(v, t, o), t.extraFieldExtendedTimestamp = v);
    const g = a.get(6534);
    g && (t.extraFieldUSDZ = g);
}

async function Jn(e, t, r, n, o) {
    const i = no(e.data), a = new We;
    a.append(o[r]);
    const s = no(new Uint8Array(4));
    s.setUint32(0, a.get(), !0);
    const u = to(i, 1);
    Object.assign(e, {
        version: Qn(i, 0),
        [t]: cn(e.data.subarray(5)),
        valid: !o.bitFlag.languageEncodingFlag && u == to(s, 0)
    }), e.valid && (n[t] = e[t], n[t + "UTF8"] = !0);
}

function Xn(e, t, r) {
    return t[r] === Pe ? e.options[r] : t[r];
}

function Zn(e) {
    const t = (4294901760 & e) >> 16, r = 65535 & e;
    try {
        return new Date(1980 + ((65024 & t) >> 9), ((480 & t) >> 5) - 1, 31 & t, (63488 & r) >> 11, (2016 & r) >> 5, 2 * (31 & r), 0);
    } catch (e) {}
}

function Yn(e) {
    return new Date(Number(e / BigInt(1e4) - BigInt(116444736e5)));
}

function Qn(e, t) {
    return e.getUint8(t);
}

function eo(e, t) {
    return e.getUint16(t, !0);
}

function to(e, t) {
    return e.getUint32(t, !0);
}

function ro(e, t) {
    return Number(e.getBigUint64(t, !0));
}

function no(e) {
    return new DataView(e.buffer);
}

const oo = "File already exists", io = "Zip file comment exceeds 64KB", ao = "File entry comment exceeds 64KB", so = "File entry name exceeds 64KB", uo = "Version exceeds 65535", lo = "The strength must equal 1, 2, or 3", co = "Extra field type exceeds 65535", fo = "Extra field data exceeds 64KB", po = "Zip64 is not supported (make sure 'keepOrder' is set to 'true')", ho = new Uint8Array([ 7, 0, 2, 0, 65, 69, 3, 0, 0 ]);

let vo = 0;

const go = [];

class mo {
    constructor(e, t = {}) {
        const r = (e = nn(e)).availableSize !== Pe && e.availableSize > 0 && e.availableSize !== 1 / 0 && e.maxSize !== Pe && e.maxSize > 0 && e.maxSize !== 1 / 0;
        Object.assign(this, {
            writer: e,
            addSplitZipSignature: r,
            options: t,
            config: Ne(),
            files: new Map,
            filenames: new Set,
            offset: e.writable.size,
            pendingEntriesSize: 0,
            pendingAddFileCalls: new Set,
            bufferedWrites: 0
        });
    }
    async add(e = "", t, r = {}) {
        const n = this, {pendingAddFileCalls: o, config: i} = n;
        let a;
        vo < i.maxWorkers ? vo++ : await new Promise(e => go.push(e));
        try {
            if (e = e.trim(), n.filenames.has(e)) {
                throw new Error(oo);
            }
            return n.filenames.add(e), a = async function(e, t, r, n) {
                t = t.trim(), n.directory && !t.endsWith(xe) ? t += xe : n.directory = t.endsWith(xe);
                const o = Eo(e, n, "encodeText", Ke);
                let i = o(t);
                i === Pe && (i = Ke(t));
                if (Co(i) > ve) {
                    throw new Error(so);
                }
                const a = n.comment || "";
                let s = o(a);
                s === Pe && (s = Ke(a));
                if (Co(s) > ve) {
                    throw new Error(ao);
                }
                const u = Eo(e, n, "version", 20);
                if (u > ve) {
                    throw new Error(uo);
                }
                const l = Eo(e, n, "versionMadeBy", 20);
                if (l > ve) {
                    throw new Error(uo);
                }
                const c = Eo(e, n, _n, new Date), f = Eo(e, n, bn), d = Eo(e, n, Dn), p = Eo(e, n, Cn, !0), h = Eo(e, n, An, 0), v = Eo(e, n, On, 0), g = Eo(e, n, "password"), m = Eo(e, n, "rawPassword"), y = Eo(e, n, "encryptionStrength", 3), _ = Eo(e, n, "zipCrypto"), E = Eo(e, n, "extendedTimestamp", !0), b = Eo(e, n, "keepOrder", !0), w = Eo(e, n, "level"), D = Eo(e, n, "useWebWorkers"), S = Eo(e, n, "bufferedWrite"), A = Eo(e, n, "dataDescriptorSignature", !1), O = Eo(e, n, "signal"), C = Eo(e, n, "useCompressionStream");
                let x = Eo(e, n, "dataDescriptor", !0), F = Eo(e, n, xn);
                if (g !== Pe && y !== Pe && (y < 1 || y > 3)) {
                    throw new Error(lo);
                }
                let M = new Uint8Array;
                const {extraField: P} = n;
                if (P) {
                    let e = 0, t = 0;
                    P.forEach(t => e += 4 + Co(t)), M = new Uint8Array(e), P.forEach((e, r) => {
                        if (r > ve) {
                            throw new Error(co);
                        }
                        if (Co(e) > ve) {
                            throw new Error(fo);
                        }
                        Ao(M, new Uint16Array([ r ]), t), Ao(M, new Uint16Array([ Co(e) ]), t + 2), Ao(M, e, t + 4), 
                        t += 4 + Co(e);
                    });
                }
                let I = 0, k = 0, R = 0;
                const T = !0 === F;
                r && (r = rn(r), await tn(r), r.size === Pe ? (x = !0, (F || F === Pe) && (F = !0, 
                R = I = 4294967296)) : (R = r.size, I = function(e) {
                    return e + 5 * (Math.floor(e / 16383) + 1);
                }(R)));
                const {diskOffset: j, diskNumber: L, maxSize: N} = e.writer, B = T || R > he, U = T || I > he, z = T || e.offset + e.pendingEntriesSize - j > he, H = Eo(e, n, "supportZip64SplitFile", !0), $ = H && T || L + Math.ceil(e.pendingEntriesSize / N) > ve;
                if (z || B || U || $) {
                    if (!1 === F || !b) {
                        throw new Error(po);
                    }
                    F = !0;
                }
                F = F || !1, n = Object.assign({}, n, {
                    rawFilename: i,
                    rawComment: s,
                    version: u,
                    versionMadeBy: l,
                    lastModDate: c,
                    lastAccessDate: f,
                    creationDate: d,
                    rawExtraField: M,
                    zip64: F,
                    zip64UncompressedSize: B,
                    zip64CompressedSize: U,
                    zip64Offset: z,
                    zip64DiskNumberStart: $,
                    password: g,
                    rawPassword: m,
                    level: C || e.config.CompressionStream !== Pe || e.config.CompressionStreamNative !== Pe ? w : 0,
                    useWebWorkers: D,
                    encryptionStrength: y,
                    extendedTimestamp: E,
                    zipCrypto: _,
                    bufferedWrite: S,
                    keepOrder: b,
                    dataDescriptor: x,
                    dataDescriptorSignature: A,
                    signal: O,
                    msDosCompatible: p,
                    internalFileAttribute: h,
                    externalFileAttribute: v,
                    useCompressionStream: C
                });
                const G = function(e) {
                    const {rawFilename: t, lastModDate: r, lastAccessDate: n, creationDate: o, rawPassword: i, password: a, level: s, zip64: u, zipCrypto: l, dataDescriptor: c, directory: f, rawExtraField: d, encryptionStrength: p, extendedTimestamp: h} = e, v = 0 !== s && !f, g = Boolean(a && Co(a) || i && Co(i));
                    let m, y, _, E, b = e.version;
                    if (g && !l) {
                        m = new Uint8Array(Co(ho) + 2);
                        const e = Oo(m);
                        wo(e, 0, Se), Ao(m, ho, 2), bo(e, 8, p);
                    } else {
                        m = new Uint8Array;
                    }
                    if (h) {
                        _ = new Uint8Array(9 + (n ? 4 : 0) + (o ? 4 : 0));
                        const e = Oo(_);
                        wo(e, 0, Ae), wo(e, 2, Co(_) - 4), E = 1 + (n ? 2 : 0) + (o ? 4 : 0), bo(e, 4, E);
                        let t = 5;
                        Do(e, t, Math.floor(r.getTime() / 1e3)), t += 4, n && (Do(e, t, Math.floor(n.getTime() / 1e3)), 
                        t += 4), o && Do(e, t, Math.floor(o.getTime() / 1e3));
                        try {
                            y = new Uint8Array(36);
                            const e = Oo(y), t = _o(r);
                            wo(e, 0, 10), wo(e, 2, 32), wo(e, 8, 1), wo(e, 10, 24), So(e, 12, t), So(e, 20, _o(n) || t), 
                            So(e, 28, _o(o) || t);
                        } catch (e) {
                            y = new Uint8Array;
                        }
                    } else {
                        y = _ = new Uint8Array;
                    }
                    let w = Ce;
                    c && (w |= 8);
                    let D = 0;
                    v && (D = 8);
                    u && (b = b > 45 ? b : 45);
                    g && (w |= 1, l || (b = b > 51 ? b : 51, D = 99, v && (m[9] = 8)));
                    const S = new Uint8Array(26), A = Oo(S);
                    wo(A, 0, b), wo(A, 2, w), wo(A, 4, D);
                    const O = new Uint32Array(1), C = Oo(O);
                    let x;
                    x = r < Me ? Me : r > Fe ? Fe : r;
                    wo(C, 0, (x.getHours() << 6 | x.getMinutes()) << 5 | x.getSeconds() / 2), wo(C, 2, (x.getFullYear() - 1980 << 4 | x.getMonth() + 1) << 5 | x.getDate());
                    const F = O[0];
                    Do(A, 6, F), wo(A, 22, Co(t));
                    const M = Co(m, _, y, d);
                    wo(A, 24, M);
                    const P = new Uint8Array(30 + Co(t) + M);
                    return Do(Oo(P), 0, ge), Ao(P, S, 4), Ao(P, t, 30), Ao(P, m, 30 + Co(t)), Ao(P, _, 30 + Co(t, m)), 
                    Ao(P, y, 30 + Co(t, m, _)), Ao(P, d, 30 + Co(t, m, _, y)), {
                        localHeaderArray: P,
                        headerArray: S,
                        headerView: A,
                        lastModDate: r,
                        rawLastModDate: F,
                        encrypted: g,
                        compressed: v,
                        version: b,
                        compressionMethod: D,
                        extraFieldExtendedTimestampFlag: E,
                        rawExtraFieldExtendedTimestamp: _,
                        rawExtraFieldNTFS: y,
                        rawExtraFieldAES: m,
                        extraFieldLength: M
                    };
                }(n), W = function(e) {
                    const {zip64: t, dataDescriptor: r, dataDescriptorSignature: n} = e;
                    let o, i = new Uint8Array, a = 0;
                    r && (i = new Uint8Array(t ? n ? 24 : 20 : n ? 16 : 12), o = Oo(i), n && (a = 4, 
                    Do(o, 0, ye)));
                    return {
                        dataDescriptorArray: i,
                        dataDescriptorView: o,
                        dataDescriptorOffset: a
                    };
                }(n), V = Co(G.localHeaderArray, W.dataDescriptorArray);
                k = V + I, e.options.usdz && (k += k + 64);
                let K;
                e.pendingEntriesSize += k;
                try {
                    K = await async function(e, t, r, n, o) {
                        const {files: i, writer: a} = e, {keepOrder: s, dataDescriptor: u, signal: l} = o, {headerInfo: c} = n, {usdz: f} = e.options, d = Array.from(i.values()).pop();
                        let p, h, v, g, m, y, _, E = {};
                        i.set(t, E);
                        try {
                            let c;
                            s && (c = d && d.lock, b()), !(o.bufferedWrite || e.writerLocked || e.bufferedWrites && s) && u || f ? (y = a, 
                            await w()) : (y = new TransformStream, _ = new Response(y.readable).blob(), y.writable.size = 0, 
                            p = !0, e.bufferedWrites++, await tn(a)), await tn(y);
                            const {writable: h} = a;
                            let {diskOffset: v} = a;
                            if (e.addSplitZipSignature) {
                                delete e.addSplitZipSignature;
                                const t = new Uint8Array(4);
                                Do(Oo(t), 0, me), await yo(h, t), e.offset += 4;
                            }
                            f && function(e, t) {
                                const {headerInfo: r} = e;
                                let {localHeaderArray: n, extraFieldLength: o} = r, i = Oo(n), a = 64 - (t + Co(n)) % 64;
                                a < 4 && (a += 64);
                                const s = new Uint8Array(a), u = Oo(s);
                                wo(u, 0, Oe), wo(u, 2, a - 2);
                                const l = n;
                                r.localHeaderArray = n = new Uint8Array(Co(l) + a), Ao(n, l), Ao(n, s, Co(l)), i = Oo(n), 
                                wo(i, 28, o + a), e.metadataSize += a;
                            }(n, e.offset - v), p || (await c, await D(h));
                            const {diskNumber: S} = a;
                            if (m = !0, E.diskNumberStart = S, E = await async function(e, t, {diskNumberStart: r, lock: n}, o, i, a) {
                                const {headerInfo: s, dataDescriptorInfo: u, metadataSize: l} = o, {localHeaderArray: c, headerArray: f, lastModDate: d, rawLastModDate: p, encrypted: h, compressed: v, version: g, compressionMethod: m, rawExtraFieldExtendedTimestamp: y, extraFieldExtendedTimestampFlag: _, rawExtraFieldNTFS: E, rawExtraFieldAES: b} = s, {dataDescriptorArray: w} = u, {rawFilename: D, lastAccessDate: S, creationDate: A, password: O, rawPassword: C, level: x, zip64: F, zip64UncompressedSize: M, zip64CompressedSize: P, zip64Offset: I, zip64DiskNumberStart: k, zipCrypto: R, dataDescriptor: T, directory: j, versionMadeBy: L, rawComment: N, rawExtraField: B, useWebWorkers: U, onstart: z, onprogress: H, onend: $, signal: G, encryptionStrength: W, extendedTimestamp: V, msDosCompatible: K, internalFileAttribute: q, externalFileAttribute: J, useCompressionStream: X} = a, Z = {
                                    lock: n,
                                    versionMadeBy: L,
                                    zip64: F,
                                    directory: Boolean(j),
                                    filenameUTF8: !0,
                                    rawFilename: D,
                                    commentUTF8: !0,
                                    rawComment: N,
                                    rawExtraFieldExtendedTimestamp: y,
                                    rawExtraFieldNTFS: E,
                                    rawExtraFieldAES: b,
                                    rawExtraField: B,
                                    extendedTimestamp: V,
                                    msDosCompatible: K,
                                    internalFileAttribute: q,
                                    externalFileAttribute: J,
                                    diskNumberStart: r
                                };
                                let Y, Q = 0, ee = 0;
                                const {writable: te} = t;
                                if (e) {
                                    e.chunkSize = Be(i), await yo(te, c);
                                    const t = e.readable, r = t.size = e.size, n = {
                                        options: {
                                            codecType: tr,
                                            level: x,
                                            rawPassword: C,
                                            password: O,
                                            encryptionStrength: W,
                                            zipCrypto: h && R,
                                            passwordVerification: h && R && p >> 8 & 255,
                                            signed: !0,
                                            compressed: v,
                                            encrypted: h,
                                            useWebWorkers: U,
                                            useCompressionStream: X,
                                            transferStreams: !1
                                        },
                                        config: i,
                                        streamOptions: {
                                            signal: G,
                                            size: r,
                                            onstart: z,
                                            onprogress: H,
                                            onend: $
                                        }
                                    }, o = await mr({
                                        readable: t,
                                        writable: te
                                    }, n);
                                    ee = o.inputSize, Q = o.outputSize, Y = o.signature, te.size += ee;
                                } else {
                                    await yo(te, c);
                                }
                                let re;
                                if (F) {
                                    let e = 4;
                                    M && (e += 8), P && (e += 8), I && (e += 8), k && (e += 4), re = new Uint8Array(e);
                                } else {
                                    re = new Uint8Array;
                                }
                                (function(e, t) {
                                    const {signature: r, rawExtraFieldZip64: n, compressedSize: o, uncompressedSize: i, headerInfo: a, dataDescriptorInfo: s} = e, {headerView: u, encrypted: l} = a, {dataDescriptorView: c, dataDescriptorOffset: f} = s, {zip64: d, zip64UncompressedSize: p, zip64CompressedSize: h, zipCrypto: v, dataDescriptor: g} = t;
                                    l && !v || r === Pe || (Do(u, 10, r), g && Do(c, f, r));
                                    if (d) {
                                        const e = Oo(n);
                                        wo(e, 0, 1), wo(e, 2, Co(n) - 4);
                                        let t = 4;
                                        p && (Do(u, 18, he), So(e, t, BigInt(i)), t += 8), h && (Do(u, 14, he), So(e, t, BigInt(o))), 
                                        g && (So(c, f + 4, BigInt(o)), So(c, f + 12, BigInt(i)));
                                    } else {
                                        Do(u, 14, o), Do(u, 18, i), g && (Do(c, f + 4, o), Do(c, f + 8, i));
                                    }
                                })({
                                    signature: Y,
                                    rawExtraFieldZip64: re,
                                    compressedSize: Q,
                                    uncompressedSize: ee,
                                    headerInfo: s,
                                    dataDescriptorInfo: u
                                }, a), T && await yo(te, w);
                                return Object.assign(Z, {
                                    uncompressedSize: ee,
                                    compressedSize: Q,
                                    lastModDate: d,
                                    rawLastModDate: p,
                                    creationDate: A,
                                    lastAccessDate: S,
                                    encrypted: h,
                                    size: l + Q,
                                    compressionMethod: m,
                                    version: g,
                                    headerArray: f,
                                    signature: Y,
                                    rawExtraFieldZip64: re,
                                    extraFieldExtendedTimestampFlag: _,
                                    zip64UncompressedSize: M,
                                    zip64CompressedSize: P,
                                    zip64Offset: I,
                                    zip64DiskNumberStart: k
                                }), Z;
                            }(r, y, E, n, e.config, o), m = !1, i.set(t, E), E.filename = t, p) {
                                await y.writable.getWriter().close();
                                let e = await _;
                                await c, await w(), g = !0, u || (e = await async function(e, t, r, {zipCrypto: n}) {
                                    let o;
                                    o = await t.slice(0, 26).arrayBuffer(), 26 != o.byteLength && (o = o.slice(0, 26));
                                    const i = new DataView(o);
                                    e.encrypted && !n || Do(i, 14, e.signature);
                                    e.zip64 ? (Do(i, 18, he), Do(i, 22, he)) : (Do(i, 18, e.compressedSize), Do(i, 22, e.uncompressedSize));
                                    return await yo(r, new Uint8Array(o)), t.slice(o.byteLength);
                                }(E, e, h, o)), await D(h), E.diskNumberStart = a.diskNumber, v = a.diskOffset, 
                                await e.stream().pipeTo(h, {
                                    preventClose: !0,
                                    preventAbort: !0,
                                    signal: l
                                }), h.size += e.size, g = !1;
                            }
                            if (E.offset = e.offset - v, E.zip64) {
                                !function(e, t) {
                                    const {rawExtraFieldZip64: r, offset: n, diskNumberStart: o} = e, {zip64UncompressedSize: i, zip64CompressedSize: a, zip64Offset: s, zip64DiskNumberStart: u} = t, l = Oo(r);
                                    let c = 4;
                                    i && (c += 8);
                                    a && (c += 8);
                                    s && (So(l, c, BigInt(n)), c += 8);
                                    u && Do(l, c, o);
                                }(E, o);
                            } else if (E.offset > he) {
                                throw new Error(po);
                            }
                            return e.offset += E.size, E;
                        } catch (r) {
                            if (p && g || !p && m) {
                                if (e.hasCorruptedEntries = !0, r) {
                                    try {
                                        r.corruptedEntry = !0;
                                    } catch (e) {}
                                }
                                p ? e.offset += y.writable.size : e.offset = y.writable.size;
                            }
                            throw i.delete(t), r;
                        } finally {
                            p && e.bufferedWrites--, v && v(), h && h();
                        }
                        function b() {
                            E.lock = new Promise(e => v = e);
                        }
                        async function w() {
                            e.writerLocked = !0;
                            const {lockWriter: t} = e;
                            e.lockWriter = new Promise(t => h = () => {
                                e.writerLocked = !1, t();
                            }), await t;
                        }
                        async function D(e) {
                            Co(c.localHeaderArray) > a.availableSize && (a.availableSize = 0, await yo(e, new Uint8Array));
                        }
                    }(e, t, r, {
                        headerInfo: G,
                        dataDescriptorInfo: W,
                        metadataSize: V
                    }, n);
                } finally {
                    e.pendingEntriesSize -= k;
                }
                return Object.assign(K, {
                    name: t,
                    comment: a,
                    extraField: P
                }), new Mn(K);
            }(n, e, t, r), o.add(a), await a;
        } catch (t) {
            throw n.filenames.delete(e), t;
        } finally {
            o.delete(a);
            const e = go.shift();
            e ? e() : vo--;
        }
    }
    async close(e = new Uint8Array, t = {}) {
        const {pendingAddFileCalls: r, writer: n} = this, {writable: o} = n;
        for (;r.size; ) {
            await Promise.allSettled(Array.from(r));
        }
        await async function(e, t, r) {
            const {files: n, writer: o} = e, {diskOffset: i, writable: a} = o;
            let {diskNumber: s} = o, u = 0, l = 0, c = e.offset - i, f = n.size;
            for (const [, e] of n) {
                const {rawFilename: t, rawExtraFieldZip64: r, rawExtraFieldAES: n, rawComment: o, rawExtraFieldNTFS: i, rawExtraField: a, extendedTimestamp: s, extraFieldExtendedTimestampFlag: u, lastModDate: c} = e;
                let f;
                if (s) {
                    f = new Uint8Array(9);
                    const e = Oo(f);
                    wo(e, 0, Ae), wo(e, 2, 5), bo(e, 4, u), Do(e, 5, Math.floor(c.getTime() / 1e3));
                } else {
                    f = new Uint8Array;
                }
                e.rawExtraFieldCDExtendedTimestamp = f, l += 46 + Co(t, o, r, n, i, f, a);
            }
            const d = new Uint8Array(l), p = Oo(d);
            await tn(o);
            let h = 0;
            for (const [e, t] of Array.from(n.values()).entries()) {
                const {offset: i, rawFilename: s, rawExtraFieldZip64: l, rawExtraFieldAES: c, rawExtraFieldCDExtendedTimestamp: f, rawExtraFieldNTFS: v, rawExtraField: g, rawComment: m, versionMadeBy: y, headerArray: _, directory: E, zip64: b, zip64UncompressedSize: w, zip64CompressedSize: D, zip64DiskNumberStart: S, zip64Offset: A, msDosCompatible: O, internalFileAttribute: C, externalFileAttribute: x, diskNumberStart: F, uncompressedSize: M, compressedSize: P} = t, I = Co(l, c, f, v, g);
                Do(p, u, _e), wo(p, u + 4, y);
                const k = Oo(_);
                w || Do(k, 18, M), D || Do(k, 14, P), Ao(d, _, u + 6), wo(p, u + 30, I), wo(p, u + 32, Co(m)), 
                wo(p, u + 34, b && S ? ve : F), wo(p, u + 36, C), x ? Do(p, u + 38, x) : E && O && bo(p, u + 38, 16), 
                Do(p, u + 42, b && A ? he : i), Ao(d, s, u + 46), Ao(d, l, u + 46 + Co(s)), Ao(d, c, u + 46 + Co(s, l)), 
                Ao(d, f, u + 46 + Co(s, l, c)), Ao(d, v, u + 46 + Co(s, l, c, f)), Ao(d, g, u + 46 + Co(s, l, c, f, v)), 
                Ao(d, m, u + 46 + Co(s) + I);
                const R = 46 + Co(s, m) + I;
                if (u - h > o.availableSize && (o.availableSize = 0, await yo(a, d.slice(h, u)), 
                h = u), u += R, r.onprogress) {
                    try {
                        await r.onprogress(e + 1, n.size, new Mn(t));
                    } catch (e) {}
                }
            }
            await yo(a, h ? d.slice(h) : d);
            let v = o.diskNumber;
            const {availableSize: g} = o;
            g < De && v++;
            let m = Eo(e, r, "zip64");
            if (c > he || l > he || f > ve || v > ve) {
                if (!1 === m) {
                    throw new Error(po);
                }
                m = !0;
            }
            const y = new Uint8Array(m ? 98 : De), _ = Oo(y);
            if (u = 0, m) {
                Do(_, 0, be), So(_, 4, BigInt(44)), wo(_, 12, 45), wo(_, 14, 45), Do(_, 16, v), 
                Do(_, 20, s), So(_, 24, BigInt(f)), So(_, 32, BigInt(f)), So(_, 40, BigInt(l)), 
                So(_, 48, BigInt(c)), Do(_, 56, we), So(_, 64, BigInt(c) + BigInt(l)), Do(_, 72, v + 1);
                Eo(e, r, "supportZip64SplitFile", !0) && (v = ve, s = ve), f = ve, c = he, l = he, 
                u += 76;
            }
            Do(_, u, Ee), wo(_, u + 4, v), wo(_, u + 6, s), wo(_, u + 8, f), wo(_, u + 10, f), 
            Do(_, u + 12, l), Do(_, u + 16, c);
            const E = Co(t);
            if (E) {
                if (!(E <= ve)) {
                    throw new Error(io);
                }
                wo(_, u + 20, E);
            }
            await yo(a, y), E && await yo(a, t);
        }(this, e, t);
        return Eo(this, t, "preventClose") || await o.getWriter().close(), n.getData ? n.getData() : o;
    }
}

async function yo(e, t) {
    const r = e.getWriter();
    try {
        await r.ready, e.size += Co(t), await r.write(t);
    } finally {
        r.releaseLock();
    }
}

function _o(e) {
    if (e) {
        return (BigInt(e.getTime()) + BigInt(116444736e5)) * BigInt(1e4);
    }
}

function Eo(e, t, r, n) {
    const o = t[r] === Pe ? e.options[r] : t[r];
    return o === Pe ? n : o;
}

function bo(e, t, r) {
    e.setUint8(t, r);
}

function wo(e, t, r) {
    e.setUint16(t, r, !0);
}

function Do(e, t, r) {
    e.setUint32(t, r, !0);
}

function So(e, t, r) {
    e.setBigUint64(t, r, !0);
}

function Ao(e, t, r) {
    e.set(t, r);
}

function Oo(e) {
    return new DataView(e.buffer);
}

function Co(...e) {
    let t = 0;
    return e.forEach(e => e && (t += e.length)), t;
}

class xo {
    constructor(e, t, r, n) {
        const o = this;
        if (e.root && n && n.getChildByName(t)) {
            throw new Error("Entry filename already exists");
        }
        r || (r = {}), Object.assign(o, {
            fs: e,
            name: t,
            data: r.data,
            options: r.options,
            id: e.entries.length,
            parent: n,
            children: [],
            uncompressedSize: r.uncompressedSize || 0
        }), e.entries.push(o), n && o.parent.children.push(o);
    }
    moveTo(e) {
        this.fs.move(this, e);
    }
    getFullname() {
        return this.getRelativeName();
    }
    getRelativeName(e = this.fs.root) {
        let t = this.name, r = this.parent;
        for (;r && r != e; ) {
            t = (r.name ? r.name + "/" : "") + t, r = r.parent;
        }
        return t;
    }
    isDescendantOf(e) {
        let t = this.parent;
        for (;t && t.id != e.id; ) {
            t = t.parent;
        }
        return Boolean(t);
    }
    rename(e) {
        const t = this.parent;
        if (t && t.getChildByName(e)) {
            throw new Error("Entry filename already exists");
        }
        this.name = e;
    }
}

class Fo extends xo {
    constructor(e, t, r, n) {
        super(e, t, r, n);
        const o = this;
        o.Reader = r.Reader, o.Writer = r.Writer, r.getData && (o.getData = r.getData);
    }
    clone() {
        return new Fo(this.fs, this.name, this);
    }
    async getData(e, t = {}) {
        const r = this;
        if (!e || e.constructor == r.Writer && r.data) {
            return r.data;
        }
        {
            const n = r.reader = new r.Reader(r.data, t), o = r.data ? r.data.uncompressedSize : n.size;
            await Promise.all([ tn(n), tn(e, o) ]);
            const i = n.readable;
            return i.size = r.uncompressedSize = n.size, await i.pipeTo(e.writable), e.getData ? e.getData() : e.writable;
        }
    }
    isPasswordProtected() {
        return this.data.encrypted;
    }
    async checkPassword(e, t = {}) {
        const r = this;
        if (!r.isPasswordProtected()) {
            return !0;
        }
        t.password = e, t.checkPasswordOnly = !0;
        try {
            return await r.data.getData(null, t), !0;
        } catch (e) {
            if (e.message == rt) {
                return !1;
            }
            throw e;
        }
    }
    getText(e, t) {
        return this.getData(new jr(e), t);
    }
    getBlob(e, t) {
        return this.getData(new Rr(e), t);
    }
    getData64URI(e, t) {
        return this.getData(new Ir(e), t);
    }
    getUint8Array(e) {
        return this.getData(new Yr, e);
    }
    getWritable(e = new WritableStream, t) {
        return this.getData({
            writable: e
        }, t);
    }
    replaceBlob(e) {
        Object.assign(this, {
            data: e,
            Reader: kr,
            Writer: Rr,
            reader: null
        });
    }
    replaceText(e) {
        Object.assign(this, {
            data: e,
            Reader: Tr,
            Writer: jr,
            reader: null
        });
    }
    replaceData64URI(e) {
        Object.assign(this, {
            data: e,
            Reader: Pr,
            Writer: Ir,
            reader: null
        });
    }
    replaceUint8Array(e) {
        Object.assign(this, {
            data: e,
            Reader: Zr,
            Writer: Yr,
            reader: null
        });
    }
    replaceReadable(e) {
        Object.assign(this, {
            data: null,
            Reader: function() {
                return {
                    readable: e
                };
            },
            Writer: null,
            reader: null
        });
    }
}

class Mo extends xo {
    constructor(e, t, r, n) {
        super(e, t, r, n), this.directory = !0;
    }
    clone(e) {
        const t = this, r = new Mo(t.fs, t.name);
        return e && (r.children = t.children.map(t => {
            const n = t.clone(e);
            return n.parent = r, n;
        })), r;
    }
    addDirectory(e, t) {
        return No(this, e, {
            options: t
        }, !0);
    }
    addText(e, t, r = {}) {
        return No(this, e, {
            data: t,
            Reader: Tr,
            Writer: jr,
            options: r,
            uncompressedSize: t.length
        });
    }
    addBlob(e, t, r = {}) {
        return No(this, e, {
            data: t,
            Reader: kr,
            Writer: Rr,
            options: r,
            uncompressedSize: t.size
        });
    }
    addData64URI(e, t, r = {}) {
        let n = t.length;
        for (;"=" == t.charAt(n - 1); ) {
            n--;
        }
        const o = t.indexOf(",") + 1;
        return No(this, e, {
            data: t,
            Reader: Pr,
            Writer: Ir,
            options: r,
            uncompressedSize: Math.floor(.75 * (n - o))
        });
    }
    addUint8Array(e, t, r = {}) {
        return No(this, e, {
            data: t,
            Reader: Zr,
            Writer: Yr,
            options: r,
            uncompressedSize: t.length
        });
    }
    addHttpContent(e, t, r = {}) {
        return No(this, e, {
            data: t,
            Reader: class extends Xr {
                constructor(e) {
                    super(e, r);
                }
            },
            options: r
        });
    }
    addReadable(e, t, r = {}) {
        return No(this, e, {
            Reader: function() {
                return {
                    readable: t
                };
            },
            options: r
        });
    }
    addFileSystemEntry(e, t = {}) {
        return To(this, e, t);
    }
    addFileSystemHandle(e, t = {}) {
        return To(this, e, t);
    }
    addFile(e, t = {}) {
        return t.lastModDate || (t.lastModDate = new Date(e.lastModified)), No(this, e.name, {
            data: e,
            Reader: function() {
                return {
                    readable: e.stream(),
                    size: e.size
                };
            },
            options: t,
            uncompressedSize: e.size
        });
    }
    addData(e, t) {
        return No(this, e, t);
    }
    importBlob(e, t) {
        return this.importZip(new kr(e), t);
    }
    importData64URI(e, t) {
        return this.importZip(new Pr(e), t);
    }
    importUint8Array(e, t) {
        return this.importZip(new Zr(e), t);
    }
    importHttpContent(e, t) {
        return this.importZip(new Xr(e, t), t);
    }
    importReadable(e, t) {
        return this.importZip({
            readable: e
        }, t);
    }
    exportBlob(e = {}) {
        return this.exportZip(new Rr(e.mimeType || "application/zip"), e);
    }
    exportData64URI(e = {}) {
        return this.exportZip(new Ir(e.mimeType || "application/zip"), e);
    }
    exportUint8Array(e = {}) {
        return this.exportZip(new Yr, e);
    }
    async exportWritable(e = new WritableStream, t = {}) {
        return await this.exportZip({
            writable: e
        }, t), e;
    }
    async importZip(e, t = {}) {
        await tn(e);
        const r = new Wn(e, t), n = [], o = await r.getEntries();
        for (const e of o) {
            let r = this;
            try {
                const o = e.filename.split("/"), i = o.pop();
                o.forEach((t, i) => {
                    const a = r;
                    r = r.getChildByName(t), r || (r = new Mo(this.fs, t, {
                        data: i == o.length - 1 ? e : null
                    }, a), n.push(r));
                }), e.directory || n.push(No(r, i, {
                    data: e,
                    Reader: Io(Object.assign({}, t)),
                    uncompressedSize: e.uncompressedSize
                }));
            } catch (t) {
                try {
                    t.cause = {
                        entry: e
                    };
                } catch (e) {}
                throw t;
            }
        }
        return n;
    }
    async exportZip(e, t) {
        const r = this;
        t.bufferedWrite === Pe && (t.bufferedWrite = !0), await Promise.all([ ko(r, t.readerOptions), tn(e) ]);
        const n = new mo(e, t);
        return await async function(e, t, r, n) {
            const o = t, i = new Map;
            async function a(e, t) {
                async function s() {
                    if (n.bufferedWrite) {
                        await Promise.allSettled(t.children.map(u));
                    } else {
                        for (const e of t.children) {
                            await u(e);
                        }
                    }
                }
                async function u(t) {
                    const s = n.relativePath ? t.getRelativeName(o) : t.getFullname();
                    let u = t.options || {}, l = {};
                    if (t.data instanceof Mn) {
                        const {externalFileAttribute: e, versionMadeBy: r, comment: n, lastModDate: o, creationDate: i, lastAccessDate: a} = t.data;
                        l = {
                            externalFileAttribute: e,
                            versionMadeBy: r,
                            comment: n,
                            lastModDate: o,
                            creationDate: i,
                            lastAccessDate: a
                        };
                    }
                    await e.add(s, t.reader, Object.assign({
                        directory: t.directory
                    }, Object.assign({}, n, l, u, {
                        onprogress: async e => {
                            if (n.onprogress) {
                                i.set(s, e);
                                try {
                                    await n.onprogress(Array.from(i.values()).reduce((e, t) => e + t), r);
                                } catch (e) {}
                            }
                        }
                    }))), await a(e, t);
                }
                await s();
            }
            await a(e, t);
        }(n, r, function(e, t) {
            let r = 0;
            return e.forEach(n), r;
            function n(e) {
                r += e[t], e.children && e.children.forEach(n);
            }
        }([ r ], "uncompressedSize"), t), await n.close(), e.getData ? e.getData() : e.writable;
    }
    getChildByName(e) {
        const t = this.children;
        for (let r = 0; r < t.length; r++) {
            const n = t[r];
            if (n.name == e) {
                return n;
            }
        }
    }
    isPasswordProtected() {
        const e = this.children;
        for (let t = 0; t < e.length; t++) {
            if (e[t].isPasswordProtected()) {
                return !0;
            }
        }
        return !1;
    }
    async checkPassword(e, t = {}) {
        const r = this.children;
        return !(await Promise.all(r.map(r => r.checkPassword(e, t)))).includes(!1);
    }
}

const Po = {
    FS: class {
        constructor() {
            Lo(this);
        }
        get children() {
            return this.root.children;
        }
        remove(e) {
            Ro(e), this.entries[e.id] = null;
        }
        move(e, t) {
            if (e == this.root) {
                throw new Error("Root directory cannot be moved");
            }
            if (!t.directory) {
                throw new Error("Target entry is not a directory");
            }
            if (t.isDescendantOf(e)) {
                throw new Error("Entry is a ancestor of target entry");
            }
            if (e != t) {
                if (t.getChildByName(e.name)) {
                    throw new Error("Entry filename already exists");
                }
                Ro(e), e.parent = t, t.children.push(e);
            }
        }
        find(e) {
            const t = e.split("/");
            let r = this.root;
            for (let e = 0; r && e < t.length; e++) {
                r = r.getChildByName(t[e]);
            }
            return r;
        }
        getById(e) {
            return this.entries[e];
        }
        getChildByName(e) {
            return this.root.getChildByName(e);
        }
        addDirectory(e, t) {
            return this.root.addDirectory(e, t);
        }
        addText(e, t, r) {
            return this.root.addText(e, t, r);
        }
        addBlob(e, t, r) {
            return this.root.addBlob(e, t, r);
        }
        addData64URI(e, t, r) {
            return this.root.addData64URI(e, t, r);
        }
        addUint8Array(e, t, r) {
            return this.root.addUint8Array(e, t, r);
        }
        addHttpContent(e, t, r) {
            return this.root.addHttpContent(e, t, r);
        }
        addReadable(e, t, r) {
            return this.root.addReadable(e, t, r);
        }
        addFileSystemEntry(e, t) {
            return this.root.addFileSystemEntry(e, t);
        }
        addFileSystemHandle(e, t) {
            return this.root.addFileSystemHandle(e, t);
        }
        addFile(e, t) {
            return this.root.addFile(e, t);
        }
        addData(e, t) {
            return this.root.addData(e, t);
        }
        importBlob(e, t) {
            return Lo(this), this.root.importBlob(e, t);
        }
        importData64URI(e, t) {
            return Lo(this), this.root.importData64URI(e, t);
        }
        importUint8Array(e, t) {
            return Lo(this), this.root.importUint8Array(e, t);
        }
        importHttpContent(e, t) {
            return Lo(this), this.root.importHttpContent(e, t);
        }
        importReadable(e, t) {
            return Lo(this), this.root.importReadable(e, t);
        }
        importZip(e, t) {
            return this.root.importZip(e, t);
        }
        exportBlob(e) {
            return this.root.exportBlob(e);
        }
        exportData64URI(e) {
            return this.root.exportData64URI(e);
        }
        exportUint8Array(e) {
            return this.root.exportUint8Array(e);
        }
        exportWritable(e, t) {
            return this.root.exportWritable(e, t);
        }
        isPasswordProtected() {
            return this.root.isPasswordProtected();
        }
        async checkPassword(e, t) {
            return this.root.checkPassword(e, t);
        }
    },
    ZipDirectoryEntry: Mo,
    ZipFileEntry: Fo
};

function Io(e) {
    return class extends Fr {
        constructor(e, t = {}) {
            super(), this.entry = e, this.options = t;
        }
        async init() {
            const t = this;
            t.size = t.entry.uncompressedSize;
            const r = await t.entry.getData(new Rr, Object.assign({}, t.options, e));
            t.data = r, t.blobReader = new kr(r), super.init();
        }
        readUint8Array(e, t) {
            return this.blobReader.readUint8Array(e, t);
        }
    };
}

async function ko(e, t) {
    e.children.length && await Promise.all(e.children.map(async e => {
        if (e.directory) {
            await ko(e, t);
        } else {
            const r = e.reader = new e.Reader(e.data, t);
            try {
                await tn(r);
            } catch (t) {
                try {
                    t.entryId = e.id, t.cause = {
                        entry: e
                    };
                } catch (e) {}
                throw t;
            }
            e.uncompressedSize = r.size;
        }
    }));
}

function Ro(e) {
    if (e.parent) {
        const t = e.parent.children;
        t.forEach((r, n) => {
            r.id == e.id && t.splice(n, 1);
        });
    }
}

async function To(e, t, r) {
    return async function e(t, n, o) {
        if (n) {
            try {
                if ((n.isFile || n.isDirectory) && (n = await jo(n)), "file" == n.kind) {
                    const e = await n.getFile();
                    o.push(t.addData(e.name, {
                        Reader: function() {
                            return {
                                readable: e.stream(),
                                size: e.size
                            };
                        },
                        options: Object.assign({}, {
                            lastModDate: new Date(e.lastModified)
                        }, r),
                        uncompressedSize: e.size
                    }));
                } else if ("directory" == n.kind) {
                    const r = t.addDirectory(n.name);
                    o.push(r);
                    for await (const t of n.values()) {
                        await e(r, t, o);
                    }
                }
            } catch (e) {
                const t = e.message + (n ? " (" + n.name + ")" : "");
                throw new Error(t);
            }
        }
        return o;
    }(e, t, []);
}

async function jo(e) {
    const t = {
        name: e.name
    };
    if (e.isFile && (t.kind = "file", t.getFile = () => new Promise((t, r) => e.file(t, r))), 
    e.isDirectory) {
        t.kind = "directory";
        const r = await async function(e) {
            const t = [];
            function r(e, n, o) {
                e.readEntries(async i => {
                    if (i.length) {
                        for (const e of i) {
                            t.push(await jo(e));
                        }
                        r(e, n, o);
                    } else {
                        n(t);
                    }
                }, o);
            }
            return await new Promise((t, n) => r(e.createReader(), t, n)), {
                [Symbol.iterator]() {
                    let e = 0;
                    return {
                        next() {
                            const r = {
                                value: t[e],
                                done: e === t.length
                            };
                            return e++, r;
                        }
                    };
                }
            };
        }(e);
        t.values = () => r;
    }
    return t;
}

function Lo(e) {
    e.entries = [], e.root = new Mo(e);
}

function No(e, t, r, n) {
    if (e.directory) {
        return n ? new Mo(e.fs, t, r, e) : new Fo(e.fs, t, r, e);
    }
    throw new Error("Parent entry is not a directory");
}

let Bo;

try {
    Bo = "undefined" == typeof document ? require("url").pathToFileURL(__filename).href : C && C.src || new URL("index.cjs", document.baseURI).href;
} catch (e) {}

Ue({
    baseURL: Bo
}), function(e) {
    const t = () => URL.createObjectURL(new Blob([ 'const{Array:e,Object:t,Number:n,Math:r,Error:s,Uint8Array:i,Uint16Array:o,Uint32Array:c,Int32Array:f,Map:a,DataView:l,Promise:u,TextEncoder:w,crypto:h,postMessage:d,TransformStream:p,ReadableStream:y,WritableStream:m,CompressionStream:b,DecompressionStream:g}=self,k=void 0,v="undefined",S="function";class z{constructor(e){return class extends p{constructor(t,n){const r=new e(n);super({transform(e,t){t.enqueue(r.append(e))},flush(e){const t=r.flush();t&&e.enqueue(t)}})}}}}const C=[];for(let e=0;256>e;e++){let t=e;for(let e=0;8>e;e++)1&t?t=t>>>1^3988292384:t>>>=1;C[e]=t}class x{constructor(e){this.t=e||-1}append(e){let t=0|this.t;for(let n=0,r=0|e.length;r>n;n++)t=t>>>8^C[255&(t^e[n])];this.t=t}get(){return~this.t}}class A extends p{constructor(){let e;const t=new x;super({transform(e,n){t.append(e),n.enqueue(e)},flush(){const n=new i(4);new l(n.buffer).setUint32(0,t.get()),e.value=n}}),e=this}}const _={concat(e,t){if(0===e.length||0===t.length)return e.concat(t);const n=e[e.length-1],r=_.i(n);return 32===r?e.concat(t):_.o(t,r,0|n,e.slice(0,e.length-1))},l(e){const t=e.length;if(0===t)return 0;const n=e[t-1];return 32*(t-1)+_.i(n)},u(e,t){if(32*e.length<t)return e;const n=(e=e.slice(0,r.ceil(t/32))).length;return t&=31,n>0&&t&&(e[n-1]=_.h(t,e[n-1]&2147483648>>t-1,1)),e},h:(e,t,n)=>32===e?t:(n?0|t:t<<32-e)+1099511627776*e,i:e=>r.round(e/1099511627776)||32,o(e,t,n,r){for(void 0===r&&(r=[]);t>=32;t-=32)r.push(n),n=0;if(0===t)return r.concat(e);for(let s=0;s<e.length;s++)r.push(n|e[s]>>>t),n=e[s]<<32-t;const s=e.length?e[e.length-1]:0,i=_.i(s);return r.push(_.h(t+i&31,t+i>32?n:r.pop(),1)),r}},I={p:{m(e){const t=_.l(e)/8,n=new i(t);let r;for(let s=0;t>s;s++)3&s||(r=e[s/4]),n[s]=r>>>24,r<<=8;return n},k(e){const t=[];let n,r=0;for(n=0;n<e.length;n++)r=r<<8|e[n],3&~n||(t.push(r),r=0);return 3&n&&t.push(_.h(8*(3&n),r)),t}}},P=class{constructor(e){const t=this;t.blockSize=512,t.v=[1732584193,4023233417,2562383102,271733878,3285377520],t.S=[1518500249,1859775393,2400959708,3395469782],e?(t.C=e.C.slice(0),t.A=e.A.slice(0),t._=e._):t.reset()}reset(){const e=this;return e.C=e.v.slice(0),e.A=[],e._=0,e}update(e){const t=this;"string"==typeof e&&(e=I.I.k(e));const n=t.A=_.concat(t.A,e),r=t._,i=t._=r+_.l(e);if(i>9007199254740991)throw new s("Cannot hash more than 2^53 - 1 bits");const o=new c(n);let f=0;for(let e=t.blockSize+r-(t.blockSize+r&t.blockSize-1);i>=e;e+=t.blockSize)t.P(o.subarray(16*f,16*(f+1))),f+=1;return n.splice(0,16*f),t}D(){const e=this;let t=e.A;const n=e.C;t=_.concat(t,[_.h(1,1)]);for(let e=t.length+2;15&e;e++)t.push(0);for(t.push(r.floor(e._/4294967296)),t.push(0|e._);t.length;)e.P(t.splice(0,16));return e.reset(),n}V(e,t,n,r){return e>19?e>39?e>59?e>79?void 0:t^n^r:t&n|t&r|n&r:t^n^r:t&n|~t&r}R(e,t){return t<<e|t>>>32-e}P(t){const n=this,s=n.C,i=e(80);for(let e=0;16>e;e++)i[e]=t[e];let o=s[0],c=s[1],f=s[2],a=s[3],l=s[4];for(let e=0;79>=e;e++){16>e||(i[e]=n.R(1,i[e-3]^i[e-8]^i[e-14]^i[e-16]));const t=n.R(5,o)+n.V(e,c,f,a)+l+i[e]+n.S[r.floor(e/20)]|0;l=a,a=f,f=n.R(30,c),c=o,o=t}s[0]=s[0]+o|0,s[1]=s[1]+c|0,s[2]=s[2]+f|0,s[3]=s[3]+a|0,s[4]=s[4]+l|0}},D={getRandomValues(e){const t=new c(e.buffer),n=e=>{let t=987654321;const n=4294967295;return()=>(t=36969*(65535&t)+(t>>16)&n,(((t<<16)+(e=18e3*(65535&e)+(e>>16)&n)&n)/4294967296+.5)*(r.random()>.5?1:-1))};for(let s,i=0;i<e.length;i+=4){const e=n(4294967296*(s||r.random()));s=987654071*e(),t[i/4]=4294967296*e()|0}return e}},V={importKey:e=>new V.B(I.p.k(e)),M(e,t,n,r){if(n=n||1e4,0>r||0>n)throw new s("invalid params to pbkdf2");const i=1+(r>>5)<<2;let o,c,f,a,u;const w=new ArrayBuffer(i),h=new l(w);let d=0;const p=_;for(t=I.p.k(t),u=1;(i||1)>d;u++){for(o=c=e.encrypt(p.concat(t,[u])),f=1;n>f;f++)for(c=e.encrypt(c),a=0;a<c.length;a++)o[a]^=c[a];for(f=0;(i||1)>d&&f<o.length;f++)h.setInt32(d,o[f]),d+=4}return w.slice(0,r/8)},B:class{constructor(e){const t=this,n=t.U=P,r=[[],[]];t.K=[new n,new n];const s=t.K[0].blockSize/32;e.length>s&&(e=(new n).update(e).D());for(let t=0;s>t;t++)r[0][t]=909522486^e[t],r[1][t]=1549556828^e[t];t.K[0].update(r[0]),t.K[1].update(r[1]),t.N=new n(t.K[0])}reset(){const e=this;e.N=new e.U(e.K[0]),e.O=!1}update(e){this.O=!0,this.N.update(e)}digest(){const e=this,t=e.N.D(),n=new e.U(e.K[1]).update(t).D();return e.reset(),n}encrypt(e){if(this.O)throw new s("encrypt on already updated hmac called!");return this.update(e),this.digest(e)}}},R=typeof h!=v&&typeof h.getRandomValues==S,B="Invalid password",E="Invalid signature",M="zipjs-abort-check-password";function U(e){return R?h.getRandomValues(e):D.getRandomValues(e)}const K=16,N={name:"PBKDF2"},O=t.assign({hash:{name:"HMAC"}},N),T=t.assign({iterations:1e3,hash:{name:"SHA-1"}},N),W=["deriveBits"],j=[8,12,16],H=[16,24,32],L=10,F=[0,0,0,0],q=typeof h!=v,G=q&&h.subtle,J=q&&typeof G!=v,Q=I.p,X=class{constructor(e){const t=this;t.T=[[[],[],[],[],[]],[[],[],[],[],[]]],t.T[0][0][0]||t.W();const n=t.T[0][4],r=t.T[1],i=e.length;let o,c,f,a=1;if(4!==i&&6!==i&&8!==i)throw new s("invalid aes key size");for(t.S=[c=e.slice(0),f=[]],o=i;4*i+28>o;o++){let e=c[o-1];(o%i==0||8===i&&o%i==4)&&(e=n[e>>>24]<<24^n[e>>16&255]<<16^n[e>>8&255]<<8^n[255&e],o%i==0&&(e=e<<8^e>>>24^a<<24,a=a<<1^283*(a>>7))),c[o]=c[o-i]^e}for(let e=0;o;e++,o--){const t=c[3&e?o:o-4];f[e]=4>=o||4>e?t:r[0][n[t>>>24]]^r[1][n[t>>16&255]]^r[2][n[t>>8&255]]^r[3][n[255&t]]}}encrypt(e){return this.j(e,0)}decrypt(e){return this.j(e,1)}W(){const e=this.T[0],t=this.T[1],n=e[4],r=t[4],s=[],i=[];let o,c,f,a;for(let e=0;256>e;e++)i[(s[e]=e<<1^283*(e>>7))^e]=e;for(let l=o=0;!n[l];l^=c||1,o=i[o]||1){let i=o^o<<1^o<<2^o<<3^o<<4;i=i>>8^255&i^99,n[l]=i,r[i]=l,a=s[f=s[c=s[l]]];let u=16843009*a^65537*f^257*c^16843008*l,w=257*s[i]^16843008*i;for(let n=0;4>n;n++)e[n][l]=w=w<<24^w>>>8,t[n][i]=u=u<<24^u>>>8}for(let n=0;5>n;n++)e[n]=e[n].slice(0),t[n]=t[n].slice(0)}j(e,t){if(4!==e.length)throw new s("invalid aes block size");const n=this.S[t],r=n.length/4-2,i=[0,0,0,0],o=this.T[t],c=o[0],f=o[1],a=o[2],l=o[3],u=o[4];let w,h,d,p=e[0]^n[0],y=e[t?3:1]^n[1],m=e[2]^n[2],b=e[t?1:3]^n[3],g=4;for(let e=0;r>e;e++)w=c[p>>>24]^f[y>>16&255]^a[m>>8&255]^l[255&b]^n[g],h=c[y>>>24]^f[m>>16&255]^a[b>>8&255]^l[255&p]^n[g+1],d=c[m>>>24]^f[b>>16&255]^a[p>>8&255]^l[255&y]^n[g+2],b=c[b>>>24]^f[p>>16&255]^a[y>>8&255]^l[255&m]^n[g+3],g+=4,p=w,y=h,m=d;for(let e=0;4>e;e++)i[t?3&-e:e]=u[p>>>24]<<24^u[y>>16&255]<<16^u[m>>8&255]<<8^u[255&b]^n[g++],w=p,p=y,y=m,m=b,b=w;return i}},Y=class{constructor(e,t){this.H=e,this.L=t,this.F=t}reset(){this.F=this.L}update(e){return this.q(this.H,e,this.F)}G(e){if(255&~(e>>24))e+=1<<24;else{let t=e>>16&255,n=e>>8&255,r=255&e;255===t?(t=0,255===n?(n=0,255===r?r=0:++r):++n):++t,e=0,e+=t<<16,e+=n<<8,e+=r}return e}J(e){0===(e[0]=this.G(e[0]))&&(e[1]=this.G(e[1]))}q(e,t,n){let r;if(!(r=t.length))return[];const s=_.l(t);for(let s=0;r>s;s+=4){this.J(n);const r=e.encrypt(n);t[s]^=r[0],t[s+1]^=r[1],t[s+2]^=r[2],t[s+3]^=r[3]}return _.u(t,s)}},Z=V.B;let $=q&&J&&typeof G.importKey==S,ee=q&&J&&typeof G.deriveBits==S;class te extends p{constructor({password:e,rawPassword:n,signed:r,encryptionStrength:o,checkPasswordOnly:c}){super({start(){t.assign(this,{ready:new u((e=>this.X=e)),password:ie(e,n),signed:r,Y:o-1,pending:new i})},async transform(e,t){const n=this,{password:r,Y:o,X:f,ready:a}=n;r?(await(async(e,t,n,r)=>{const i=await se(e,t,n,ce(r,0,j[t])),o=ce(r,j[t]);if(i[0]!=o[0]||i[1]!=o[1])throw new s(B)})(n,o,r,ce(e,0,j[o]+2)),e=ce(e,j[o]+2),c?t.error(new s(M)):f()):await a;const l=new i(e.length-L-(e.length-L)%K);t.enqueue(re(n,e,l,0,L,!0))},async flush(e){const{signed:t,Z:n,$:r,pending:o,ready:c}=this;if(r&&n){await c;const f=ce(o,0,o.length-L),a=ce(o,o.length-L);let l=new i;if(f.length){const e=ae(Q,f);r.update(e);const t=n.update(e);l=fe(Q,t)}if(t){const e=ce(fe(Q,r.digest()),0,L);for(let t=0;L>t;t++)if(e[t]!=a[t])throw new s(E)}e.enqueue(l)}}})}}class ne extends p{constructor({password:e,rawPassword:n,encryptionStrength:r}){let s;super({start(){t.assign(this,{ready:new u((e=>this.X=e)),password:ie(e,n),Y:r-1,pending:new i})},async transform(e,t){const n=this,{password:r,Y:s,X:o,ready:c}=n;let f=new i;r?(f=await(async(e,t,n)=>{const r=U(new i(j[t]));return oe(r,await se(e,t,n,r))})(n,s,r),o()):await c;const a=new i(f.length+e.length-e.length%K);a.set(f,0),t.enqueue(re(n,e,a,f.length,0))},async flush(e){const{Z:t,$:n,pending:r,ready:o}=this;if(n&&t){await o;let c=new i;if(r.length){const e=t.update(ae(Q,r));n.update(e),c=fe(Q,e)}s.signature=fe(Q,n.digest()).slice(0,L),e.enqueue(oe(c,s.signature))}}}),s=this}}function re(e,t,n,r,s,o){const{Z:c,$:f,pending:a}=e,l=t.length-s;let u;for(a.length&&(t=oe(a,t),n=((e,t)=>{if(t&&t>e.length){const n=e;(e=new i(t)).set(n,0)}return e})(n,l-l%K)),u=0;l-K>=u;u+=K){const e=ae(Q,ce(t,u,u+K));o&&f.update(e);const s=c.update(e);o||f.update(s),n.set(fe(Q,s),u+r)}return e.pending=ce(t,u),n}async function se(n,r,s,o){n.password=null;const c=await(async(e,t,n,r,s)=>{if(!$)return V.importKey(t);try{return await G.importKey("raw",t,n,!1,s)}catch(e){return $=!1,V.importKey(t)}})(0,s,O,0,W),f=await(async(e,t,n)=>{if(!ee)return V.M(t,e.salt,T.iterations,n);try{return await G.deriveBits(e,t,n)}catch(r){return ee=!1,V.M(t,e.salt,T.iterations,n)}})(t.assign({salt:o},T),c,8*(2*H[r]+2)),a=new i(f),l=ae(Q,ce(a,0,H[r])),u=ae(Q,ce(a,H[r],2*H[r])),w=ce(a,2*H[r]);return t.assign(n,{keys:{key:l,ee:u,passwordVerification:w},Z:new Y(new X(l),e.from(F)),$:new Z(u)}),w}function ie(e,t){return t===k?(e=>{if(typeof w==v){const t=new i((e=unescape(encodeURIComponent(e))).length);for(let n=0;n<t.length;n++)t[n]=e.charCodeAt(n);return t}return(new w).encode(e)})(e):t}function oe(e,t){let n=e;return e.length+t.length&&(n=new i(e.length+t.length),n.set(e,0),n.set(t,e.length)),n}function ce(e,t,n){return e.subarray(t,n)}function fe(e,t){return e.m(t)}function ae(e,t){return e.k(t)}class le extends p{constructor({password:e,passwordVerification:n,checkPasswordOnly:r}){super({start(){t.assign(this,{password:e,passwordVerification:n}),de(this,e)},transform(e,t){const n=this;if(n.password){const t=we(n,e.subarray(0,12));if(n.password=null,t[11]!=n.passwordVerification)throw new s(B);e=e.subarray(12)}r?t.error(new s(M)):t.enqueue(we(n,e))}})}}class ue extends p{constructor({password:e,passwordVerification:n}){super({start(){t.assign(this,{password:e,passwordVerification:n}),de(this,e)},transform(e,t){const n=this;let r,s;if(n.password){n.password=null;const t=U(new i(12));t[11]=n.passwordVerification,r=new i(e.length+t.length),r.set(he(n,t),0),s=12}else r=new i(e.length),s=0;r.set(he(n,e),s),t.enqueue(r)}})}}function we(e,t){const n=new i(t.length);for(let r=0;r<t.length;r++)n[r]=ye(e)^t[r],pe(e,n[r]);return n}function he(e,t){const n=new i(t.length);for(let r=0;r<t.length;r++)n[r]=ye(e)^t[r],pe(e,t[r]);return n}function de(e,n){const r=[305419896,591751049,878082192];t.assign(e,{keys:r,te:new x(r[0]),ne:new x(r[2])});for(let t=0;t<n.length;t++)pe(e,n.charCodeAt(t))}function pe(e,t){let[n,s,i]=e.keys;e.te.append([t]),n=~e.te.get(),s=be(r.imul(be(s+me(n)),134775813)+1),e.ne.append([s>>>24]),i=~e.ne.get(),e.keys=[n,s,i]}function ye(e){const t=2|e.keys[2];return me(r.imul(t,1^t)>>>8)}function me(e){return 255&e}function be(e){return 4294967295&e}const ge="deflate-raw";class ke extends p{constructor(e,{chunkSize:t,CompressionStream:n,CompressionStreamNative:r}){super({});const{compressed:s,encrypted:i,useCompressionStream:o,zipCrypto:c,signed:f,level:a}=e,u=this;let w,h,d=Se(super.readable);i&&!c||!f||(w=new A,d=xe(d,w)),s&&(d=Ce(d,o,{level:a,chunkSize:t},r,n)),i&&(c?d=xe(d,new ue(e)):(h=new ne(e),d=xe(d,h))),ze(u,d,(()=>{let e;i&&!c&&(e=h.signature),i&&!c||!f||(e=new l(w.value.buffer).getUint32(0)),u.signature=e}))}}class ve extends p{constructor(e,{chunkSize:t,DecompressionStream:n,DecompressionStreamNative:r}){super({});const{zipCrypto:i,encrypted:o,signed:c,signature:f,compressed:a,useCompressionStream:u}=e;let w,h,d=Se(super.readable);o&&(i?d=xe(d,new le(e)):(h=new te(e),d=xe(d,h))),a&&(d=Ce(d,u,{chunkSize:t},r,n)),o&&!i||!c||(w=new A,d=xe(d,w)),ze(this,d,(()=>{if((!o||i)&&c){const e=new l(w.value.buffer);if(f!=e.getUint32(0,!1))throw new s(E)}}))}}function Se(e){return xe(e,new p({transform(e,t){e&&e.length&&t.enqueue(e)}}))}function ze(e,n,r){n=xe(n,new p({flush:r})),t.defineProperty(e,"readable",{get:()=>n})}function Ce(e,t,n,r,s){try{e=xe(e,new(t&&r?r:s)(ge,n))}catch(r){if(!t)return e;try{e=xe(e,new s(ge,n))}catch(t){return e}}return e}function xe(e,t){return e.pipeThrough(t)}const Ae="data",_e="close";class Ie extends p{constructor(e,n){super({});const r=this,{codecType:s}=e;let i;s.startsWith("deflate")?i=ke:s.startsWith("inflate")&&(i=ve);let o=0,c=0;const f=new i(e,n),a=super.readable,l=new p({transform(e,t){e&&e.length&&(c+=e.length,t.enqueue(e))},flush(){t.assign(r,{inputSize:c})}}),u=new p({transform(e,t){e&&e.length&&(o+=e.length,t.enqueue(e))},flush(){const{signature:e}=f;t.assign(r,{signature:e,outputSize:o,inputSize:c})}});t.defineProperty(r,"readable",{get:()=>a.pipeThrough(l).pipeThrough(f).pipeThrough(u)})}}class Pe extends p{constructor(e){let t;super({transform:function n(r,s){if(t){const e=new i(t.length+r.length);e.set(t),e.set(r,t.length),r=e,t=null}r.length>e?(s.enqueue(r.slice(0,e)),n(r.slice(e),s)):t=r},flush(e){t&&t.length&&e.enqueue(t)}})}}const De=new a,Ve=new a;let Re,Be=0,Ee=!0;async function Me(e){try{const{options:t,scripts:r,config:s}=e;if(r&&r.length)try{Ee?importScripts.apply(k,r):await Ue(r)}catch(e){Ee=!1,await Ue(r)}self.initCodec&&self.initCodec(),s.CompressionStreamNative=self.CompressionStream,s.DecompressionStreamNative=self.DecompressionStream,self.Deflate&&(s.CompressionStream=new z(self.Deflate)),self.Inflate&&(s.DecompressionStream=new z(self.Inflate));const i={highWaterMark:1},o=e.readable||new y({async pull(e){const t=new u((e=>De.set(Be,e)));Ke({type:"pull",messageId:Be}),Be=(Be+1)%n.MAX_SAFE_INTEGER;const{value:r,done:s}=await t;e.enqueue(r),s&&e.close()}},i),c=e.writable||new m({async write(e){let t;const r=new u((e=>t=e));Ve.set(Be,t),Ke({type:Ae,value:e,messageId:Be}),Be=(Be+1)%n.MAX_SAFE_INTEGER,await r}},i),f=new Ie(t,s);Re=new AbortController;const{signal:a}=Re;await o.pipeThrough(f).pipeThrough(new Pe(s.chunkSize)).pipeTo(c,{signal:a,preventClose:!0,preventAbort:!0}),await c.getWriter().close();const{signature:l,inputSize:w,outputSize:h}=f;Ke({type:_e,result:{signature:l,inputSize:w,outputSize:h}})}catch(e){Ne(e)}}async function Ue(e){for(const t of e)await import(t)}function Ke(e){let{value:t}=e;if(t)if(t.length)try{t=new i(t),e.value=t.buffer,d(e,[e.value])}catch(t){d(e)}else d(e);else d(e)}function Ne(e=new s("Unknown error")){const{message:t,stack:n,code:r,name:i}=e;d({error:{message:t,stack:n,code:r,name:i}})}addEventListener("message",(({data:e})=>{const{type:t,messageId:n,value:r,done:s}=e;try{if("start"==t&&Me(e),t==Ae){const e=De.get(n);De.delete(n),e({value:new i(r),done:s})}if("ack"==t){const e=Ve.get(n);Ve.delete(n),e()}t==_e&&Re.abort()}catch(e){Ne(e)}}));const Oe=15,Te=573,We=-2;function je(t){return He(t.map((([t,n])=>new e(t).fill(n,0,t))))}function He(t){return t.reduce(((t,n)=>t.concat(e.isArray(n)?He(n):n)),[])}const Le=[0,1,2,3].concat(...je([[2,4],[2,5],[4,6],[4,7],[8,8],[8,9],[16,10],[16,11],[32,12],[32,13],[64,14],[64,15],[2,0],[1,16],[1,17],[2,18],[2,19],[4,20],[4,21],[8,22],[8,23],[16,24],[16,25],[32,26],[32,27],[64,28],[64,29]]));function Fe(){const e=this;function t(e,t){let n=0;do{n|=1&e,e>>>=1,n<<=1}while(--t>0);return n>>>1}e.re=n=>{const s=e.se,i=e.oe.ie,o=e.oe.ce;let c,f,a,l=-1;for(n.fe=0,n.ae=Te,c=0;o>c;c++)0!==s[2*c]?(n.le[++n.fe]=l=c,n.ue[c]=0):s[2*c+1]=0;for(;2>n.fe;)a=n.le[++n.fe]=2>l?++l:0,s[2*a]=1,n.ue[a]=0,n.we--,i&&(n.he-=i[2*a+1]);for(e.de=l,c=r.floor(n.fe/2);c>=1;c--)n.pe(s,c);a=o;do{c=n.le[1],n.le[1]=n.le[n.fe--],n.pe(s,1),f=n.le[1],n.le[--n.ae]=c,n.le[--n.ae]=f,s[2*a]=s[2*c]+s[2*f],n.ue[a]=r.max(n.ue[c],n.ue[f])+1,s[2*c+1]=s[2*f+1]=a,n.le[1]=a++,n.pe(s,1)}while(n.fe>=2);n.le[--n.ae]=n.le[1],(t=>{const n=e.se,r=e.oe.ie,s=e.oe.ye,i=e.oe.me,o=e.oe.be;let c,f,a,l,u,w,h=0;for(l=0;Oe>=l;l++)t.ge[l]=0;for(n[2*t.le[t.ae]+1]=0,c=t.ae+1;Te>c;c++)f=t.le[c],l=n[2*n[2*f+1]+1]+1,l>o&&(l=o,h++),n[2*f+1]=l,f>e.de||(t.ge[l]++,u=0,i>f||(u=s[f-i]),w=n[2*f],t.we+=w*(l+u),r&&(t.he+=w*(r[2*f+1]+u)));if(0!==h){do{for(l=o-1;0===t.ge[l];)l--;t.ge[l]--,t.ge[l+1]+=2,t.ge[o]--,h-=2}while(h>0);for(l=o;0!==l;l--)for(f=t.ge[l];0!==f;)a=t.le[--c],a>e.de||(n[2*a+1]!=l&&(t.we+=(l-n[2*a+1])*n[2*a],n[2*a+1]=l),f--)}})(n),((e,n,r)=>{const s=[];let i,o,c,f=0;for(i=1;Oe>=i;i++)s[i]=f=f+r[i-1]<<1;for(o=0;n>=o;o++)c=e[2*o+1],0!==c&&(e[2*o]=t(s[c]++,c))})(s,e.de,n.ge)}}function qe(e,t,n,r,s){const i=this;i.ie=e,i.ye=t,i.me=n,i.ce=r,i.be=s}Fe.ke=[0,1,2,3,4,5,6,7].concat(...je([[2,8],[2,9],[2,10],[2,11],[4,12],[4,13],[4,14],[4,15],[8,16],[8,17],[8,18],[8,19],[16,20],[16,21],[16,22],[16,23],[32,24],[32,25],[32,26],[31,27],[1,28]])),Fe.ve=[0,1,2,3,4,5,6,7,8,10,12,14,16,20,24,28,32,40,48,56,64,80,96,112,128,160,192,224,0],Fe.Se=[0,1,2,3,4,6,8,12,16,24,32,48,64,96,128,192,256,384,512,768,1024,1536,2048,3072,4096,6144,8192,12288,16384,24576],Fe.ze=e=>256>e?Le[e]:Le[256+(e>>>7)],Fe.Ce=[0,0,0,0,0,0,0,0,1,1,1,1,2,2,2,2,3,3,3,3,4,4,4,4,5,5,5,5,0],Fe.xe=[0,0,0,0,1,1,2,2,3,3,4,4,5,5,6,6,7,7,8,8,9,9,10,10,11,11,12,12,13,13],Fe.Ae=[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,2,3,7],Fe._e=[16,17,18,0,8,7,9,6,10,5,11,4,12,3,13,2,14,1,15];const Ge=je([[144,8],[112,9],[24,7],[8,8]]);qe.Ie=He([12,140,76,204,44,172,108,236,28,156,92,220,60,188,124,252,2,130,66,194,34,162,98,226,18,146,82,210,50,178,114,242,10,138,74,202,42,170,106,234,26,154,90,218,58,186,122,250,6,134,70,198,38,166,102,230,22,150,86,214,54,182,118,246,14,142,78,206,46,174,110,238,30,158,94,222,62,190,126,254,1,129,65,193,33,161,97,225,17,145,81,209,49,177,113,241,9,137,73,201,41,169,105,233,25,153,89,217,57,185,121,249,5,133,69,197,37,165,101,229,21,149,85,213,53,181,117,245,13,141,77,205,45,173,109,237,29,157,93,221,61,189,125,253,19,275,147,403,83,339,211,467,51,307,179,435,115,371,243,499,11,267,139,395,75,331,203,459,43,299,171,427,107,363,235,491,27,283,155,411,91,347,219,475,59,315,187,443,123,379,251,507,7,263,135,391,71,327,199,455,39,295,167,423,103,359,231,487,23,279,151,407,87,343,215,471,55,311,183,439,119,375,247,503,15,271,143,399,79,335,207,463,47,303,175,431,111,367,239,495,31,287,159,415,95,351,223,479,63,319,191,447,127,383,255,511,0,64,32,96,16,80,48,112,8,72,40,104,24,88,56,120,4,68,36,100,20,84,52,116,3,131,67,195,35,163,99,227].map(((e,t)=>[e,Ge[t]])));const Je=je([[30,5]]);function Qe(e,t,n,r,s){const i=this;i.Pe=e,i.De=t,i.Ve=n,i.Re=r,i.Be=s}qe.Ee=He([0,16,8,24,4,20,12,28,2,18,10,26,6,22,14,30,1,17,9,25,5,21,13,29,3,19,11,27,7,23].map(((e,t)=>[e,Je[t]]))),qe.Me=new qe(qe.Ie,Fe.Ce,257,286,Oe),qe.Ue=new qe(qe.Ee,Fe.xe,0,30,Oe),qe.Ke=new qe(null,Fe.Ae,0,19,7);const Xe=[new Qe(0,0,0,0,0),new Qe(4,4,8,4,1),new Qe(4,5,16,8,1),new Qe(4,6,32,32,1),new Qe(4,4,16,16,2),new Qe(8,16,32,32,2),new Qe(8,16,128,128,2),new Qe(8,32,128,256,2),new Qe(32,128,258,1024,2),new Qe(32,258,258,4096,2)],Ye=["need dictionary","stream end","","","stream error","data error","","buffer error","",""],Ze=113,$e=666,et=262;function tt(e,t,n,r){const s=e[2*t],i=e[2*n];return i>s||s==i&&r[t]<=r[n]}function nt(){const e=this;let t,n,s,c,f,a,l,u,w,h,d,p,y,m,b,g,k,v,S,z,C,x,A,_,I,P,D,V,R,B,E,M,U;const K=new Fe,N=new Fe,O=new Fe;let T,W,j,H,L,F;function q(){let t;for(t=0;286>t;t++)E[2*t]=0;for(t=0;30>t;t++)M[2*t]=0;for(t=0;19>t;t++)U[2*t]=0;E[512]=1,e.we=e.he=0,W=j=0}function G(e,t){let n,r=-1,s=e[1],i=0,o=7,c=4;0===s&&(o=138,c=3),e[2*(t+1)+1]=65535;for(let f=0;t>=f;f++)n=s,s=e[2*(f+1)+1],++i<o&&n==s||(c>i?U[2*n]+=i:0!==n?(n!=r&&U[2*n]++,U[32]++):i>10?U[36]++:U[34]++,i=0,r=n,0===s?(o=138,c=3):n==s?(o=6,c=3):(o=7,c=4))}function J(t){e.Ne[e.pending++]=t}function Q(e){J(255&e),J(e>>>8&255)}function X(e,t){let n;const r=t;F>16-r?(n=e,L|=n<<F&65535,Q(L),L=n>>>16-F,F+=r-16):(L|=e<<F&65535,F+=r)}function Y(e,t){const n=2*e;X(65535&t[n],65535&t[n+1])}function Z(e,t){let n,r,s=-1,i=e[1],o=0,c=7,f=4;for(0===i&&(c=138,f=3),n=0;t>=n;n++)if(r=i,i=e[2*(n+1)+1],++o>=c||r!=i){if(f>o)do{Y(r,U)}while(0!=--o);else 0!==r?(r!=s&&(Y(r,U),o--),Y(16,U),X(o-3,2)):o>10?(Y(18,U),X(o-11,7)):(Y(17,U),X(o-3,3));o=0,s=r,0===i?(c=138,f=3):r==i?(c=6,f=3):(c=7,f=4)}}function $(){16==F?(Q(L),L=0,F=0):8>F||(J(255&L),L>>>=8,F-=8)}function ee(t,n){let s,i,o;if(e.Oe[W]=t,e.Te[W]=255&n,W++,0===t?E[2*n]++:(j++,t--,E[2*(Fe.ke[n]+256+1)]++,M[2*Fe.ze(t)]++),!(8191&W)&&D>2){for(s=8*W,i=C-k,o=0;30>o;o++)s+=M[2*o]*(5+Fe.xe[o]);if(s>>>=3,j<r.floor(W/2)&&s<r.floor(i/2))return!0}return W==T-1}function te(t,n){let r,s,i,o,c=0;if(0!==W)do{r=e.Oe[c],s=e.Te[c],c++,0===r?Y(s,t):(i=Fe.ke[s],Y(i+256+1,t),o=Fe.Ce[i],0!==o&&(s-=Fe.ve[i],X(s,o)),r--,i=Fe.ze(r),Y(i,n),o=Fe.xe[i],0!==o&&(r-=Fe.Se[i],X(r,o)))}while(W>c);Y(256,t),H=t[513]}function ne(){F>8?Q(L):F>0&&J(255&L),L=0,F=0}function re(t,n,r){X(0+(r?1:0),3),((t,n)=>{ne(),H=8,Q(n),Q(~n),e.Ne.set(u.subarray(t,t+n),e.pending),e.pending+=n})(t,n)}function se(n){((t,n,r)=>{let s,i,o=0;D>0?(K.re(e),N.re(e),o=(()=>{let t;for(G(E,K.de),G(M,N.de),O.re(e),t=18;t>=3&&0===U[2*Fe._e[t]+1];t--);return e.we+=14+3*(t+1),t})(),s=e.we+3+7>>>3,i=e.he+3+7>>>3,i>s||(s=i)):s=i=n+5,n+4>s||-1==t?i==s?(X(2+(r?1:0),3),te(qe.Ie,qe.Ee)):(X(4+(r?1:0),3),((e,t,n)=>{let r;for(X(e-257,5),X(t-1,5),X(n-4,4),r=0;n>r;r++)X(U[2*Fe._e[r]+1],3);Z(E,e-1),Z(M,t-1)})(K.de+1,N.de+1,o+1),te(E,M)):re(t,n,r),q(),r&&ne()})(0>k?-1:k,C-k,n),k=C,t.We()}function ie(){let e,n,r,s;do{if(s=w-A-C,0===s&&0===C&&0===A)s=f;else if(-1==s)s--;else if(C>=f+f-et){u.set(u.subarray(f,f+f),0),x-=f,C-=f,k-=f,e=y,r=e;do{n=65535&d[--r],d[r]=f>n?0:n-f}while(0!=--e);e=f,r=e;do{n=65535&h[--r],h[r]=f>n?0:n-f}while(0!=--e);s+=f}if(0===t.je)return;e=t.He(u,C+A,s),A+=e,3>A||(p=255&u[C],p=(p<<g^255&u[C+1])&b)}while(et>A&&0!==t.je)}function oe(e){let t,n,r=I,s=C,i=_;const o=C>f-et?C-(f-et):0;let c=B;const a=l,w=C+258;let d=u[s+i-1],p=u[s+i];R>_||(r>>=2),c>A&&(c=A);do{if(t=e,u[t+i]==p&&u[t+i-1]==d&&u[t]==u[s]&&u[++t]==u[s+1]){s+=2,t++;do{}while(u[++s]==u[++t]&&u[++s]==u[++t]&&u[++s]==u[++t]&&u[++s]==u[++t]&&u[++s]==u[++t]&&u[++s]==u[++t]&&u[++s]==u[++t]&&u[++s]==u[++t]&&w>s);if(n=258-(w-s),s=w-258,n>i){if(x=e,i=n,n>=c)break;d=u[s+i-1],p=u[s+i]}}}while((e=65535&h[e&a])>o&&0!=--r);return i>A?A:i}e.ue=[],e.ge=[],e.le=[],E=[],M=[],U=[],e.pe=(t,n)=>{const r=e.le,s=r[n];let i=n<<1;for(;i<=e.fe&&(i<e.fe&&tt(t,r[i+1],r[i],e.ue)&&i++,!tt(t,s,r[i],e.ue));)r[n]=r[i],n=i,i<<=1;r[n]=s},e.Le=(t,S,x,W,j,G)=>(W||(W=8),j||(j=8),G||(G=0),t.Fe=null,-1==S&&(S=6),1>j||j>9||8!=W||9>x||x>15||0>S||S>9||0>G||G>2?We:(t.qe=e,a=x,f=1<<a,l=f-1,m=j+7,y=1<<m,b=y-1,g=r.floor((m+3-1)/3),u=new i(2*f),h=[],d=[],T=1<<j+6,e.Ne=new i(4*T),s=4*T,e.Oe=new o(T),e.Te=new i(T),D=S,V=G,(t=>(t.Ge=t.Je=0,t.Fe=null,e.pending=0,e.Qe=0,n=Ze,c=0,K.se=E,K.oe=qe.Me,N.se=M,N.oe=qe.Ue,O.se=U,O.oe=qe.Ke,L=0,F=0,H=8,q(),(()=>{w=2*f,d[y-1]=0;for(let e=0;y-1>e;e++)d[e]=0;P=Xe[D].De,R=Xe[D].Pe,B=Xe[D].Ve,I=Xe[D].Re,C=0,k=0,A=0,v=_=2,z=0,p=0})(),0))(t))),e.Xe=()=>42!=n&&n!=Ze&&n!=$e?We:(e.Te=null,e.Oe=null,e.Ne=null,d=null,h=null,u=null,e.qe=null,n==Ze?-3:0),e.Ye=(e,t,n)=>{let r=0;return-1==t&&(t=6),0>t||t>9||0>n||n>2?We:(Xe[D].Be!=Xe[t].Be&&0!==e.Ge&&(r=e.Ze(1)),D!=t&&(D=t,P=Xe[D].De,R=Xe[D].Pe,B=Xe[D].Ve,I=Xe[D].Re),V=n,r)},e.$e=(e,t,r)=>{let s,i=r,o=0;if(!t||42!=n)return We;if(3>i)return 0;for(i>f-et&&(i=f-et,o=r-i),u.set(t.subarray(o,o+i),0),C=i,k=i,p=255&u[0],p=(p<<g^255&u[1])&b,s=0;i-3>=s;s++)p=(p<<g^255&u[s+2])&b,h[s&l]=d[p],d[p]=s;return 0},e.Ze=(r,i)=>{let o,w,m,I,R;if(i>4||0>i)return We;if(!r.et||!r.tt&&0!==r.je||n==$e&&4!=i)return r.Fe=Ye[4],We;if(0===r.nt)return r.Fe=Ye[7],-5;var B;if(t=r,I=c,c=i,42==n&&(w=8+(a-8<<4)<<8,m=(D-1&255)>>1,m>3&&(m=3),w|=m<<6,0!==C&&(w|=32),w+=31-w%31,n=Ze,J((B=w)>>8&255),J(255&B)),0!==e.pending){if(t.We(),0===t.nt)return c=-1,0}else if(0===t.je&&I>=i&&4!=i)return t.Fe=Ye[7],-5;if(n==$e&&0!==t.je)return r.Fe=Ye[7],-5;if(0!==t.je||0!==A||0!=i&&n!=$e){switch(R=-1,Xe[D].Be){case 0:R=(e=>{let n,r=65535;for(r>s-5&&(r=s-5);;){if(1>=A){if(ie(),0===A&&0==e)return 0;if(0===A)break}if(C+=A,A=0,n=k+r,(0===C||C>=n)&&(A=C-n,C=n,se(!1),0===t.nt))return 0;if(C-k>=f-et&&(se(!1),0===t.nt))return 0}return se(4==e),0===t.nt?4==e?2:0:4==e?3:1})(i);break;case 1:R=(e=>{let n,r=0;for(;;){if(et>A){if(ie(),et>A&&0==e)return 0;if(0===A)break}if(3>A||(p=(p<<g^255&u[C+2])&b,r=65535&d[p],h[C&l]=d[p],d[p]=C),0===r||(C-r&65535)>f-et||2!=V&&(v=oe(r)),3>v)n=ee(0,255&u[C]),A--,C++;else if(n=ee(C-x,v-3),A-=v,v>P||3>A)C+=v,v=0,p=255&u[C],p=(p<<g^255&u[C+1])&b;else{v--;do{C++,p=(p<<g^255&u[C+2])&b,r=65535&d[p],h[C&l]=d[p],d[p]=C}while(0!=--v);C++}if(n&&(se(!1),0===t.nt))return 0}return se(4==e),0===t.nt?4==e?2:0:4==e?3:1})(i);break;case 2:R=(e=>{let n,r,s=0;for(;;){if(et>A){if(ie(),et>A&&0==e)return 0;if(0===A)break}if(3>A||(p=(p<<g^255&u[C+2])&b,s=65535&d[p],h[C&l]=d[p],d[p]=C),_=v,S=x,v=2,0!==s&&P>_&&f-et>=(C-s&65535)&&(2!=V&&(v=oe(s)),5>=v&&(1==V||3==v&&C-x>4096)&&(v=2)),3>_||v>_)if(0!==z){if(n=ee(0,255&u[C-1]),n&&se(!1),C++,A--,0===t.nt)return 0}else z=1,C++,A--;else{r=C+A-3,n=ee(C-1-S,_-3),A-=_-1,_-=2;do{++C>r||(p=(p<<g^255&u[C+2])&b,s=65535&d[p],h[C&l]=d[p],d[p]=C)}while(0!=--_);if(z=0,v=2,C++,n&&(se(!1),0===t.nt))return 0}}return 0!==z&&(n=ee(0,255&u[C-1]),z=0),se(4==e),0===t.nt?4==e?2:0:4==e?3:1})(i)}if(2!=R&&3!=R||(n=$e),0==R||2==R)return 0===t.nt&&(c=-1),0;if(1==R){if(1==i)X(2,3),Y(256,qe.Ie),$(),9>1+H+10-F&&(X(2,3),Y(256,qe.Ie),$()),H=7;else if(re(0,0,!1),3==i)for(o=0;y>o;o++)d[o]=0;if(t.We(),0===t.nt)return c=-1,0}}return 4!=i?0:1}}function rt(){const e=this;e.rt=0,e.st=0,e.je=0,e.Ge=0,e.nt=0,e.Je=0}function st(e){const t=new rt,n=(o=e&&e.chunkSize?e.chunkSize:65536)+5*(r.floor(o/16383)+1);var o;const c=new i(n);let f=e?e.level:-1;void 0===f&&(f=-1),t.Le(f),t.et=c,this.append=(e,r)=>{let o,f,a=0,l=0,u=0;const w=[];if(e.length){t.rt=0,t.tt=e,t.je=e.length;do{if(t.st=0,t.nt=n,o=t.Ze(0),0!=o)throw new s("deflating: "+t.Fe);t.st&&(t.st==n?w.push(new i(c)):w.push(c.subarray(0,t.st))),u+=t.st,r&&t.rt>0&&t.rt!=a&&(r(t.rt),a=t.rt)}while(t.je>0||0===t.nt);return w.length>1?(f=new i(u),w.forEach((e=>{f.set(e,l),l+=e.length}))):f=w[0]?new i(w[0]):new i,f}},this.flush=()=>{let e,r,o=0,f=0;const a=[];do{if(t.st=0,t.nt=n,e=t.Ze(4),1!=e&&0!=e)throw new s("deflating: "+t.Fe);n-t.nt>0&&a.push(c.slice(0,t.st)),f+=t.st}while(t.je>0||0===t.nt);return t.Xe(),r=new i(f),a.forEach((e=>{r.set(e,o),o+=e.length})),r}}rt.prototype={Le(e,t){const n=this;return n.qe=new nt,t||(t=Oe),n.qe.Le(n,e,t)},Ze(e){const t=this;return t.qe?t.qe.Ze(t,e):We},Xe(){const e=this;if(!e.qe)return We;const t=e.qe.Xe();return e.qe=null,t},Ye(e,t){const n=this;return n.qe?n.qe.Ye(n,e,t):We},$e(e,t){const n=this;return n.qe?n.qe.$e(n,e,t):We},He(e,t,n){const r=this;let s=r.je;return s>n&&(s=n),0===s?0:(r.je-=s,e.set(r.tt.subarray(r.rt,r.rt+s),t),r.rt+=s,r.Ge+=s,s)},We(){const e=this;let t=e.qe.pending;t>e.nt&&(t=e.nt),0!==t&&(e.et.set(e.qe.Ne.subarray(e.qe.Qe,e.qe.Qe+t),e.st),e.st+=t,e.qe.Qe+=t,e.Je+=t,e.nt-=t,e.qe.pending-=t,0===e.qe.pending&&(e.qe.Qe=0))}};const it=0,ot=1,ct=-2,ft=-3,at=-4,lt=-5,ut=[0,1,3,7,15,31,63,127,255,511,1023,2047,4095,8191,16383,32767,65535],wt=1440,ht=[96,7,256,0,8,80,0,8,16,84,8,115,82,7,31,0,8,112,0,8,48,0,9,192,80,7,10,0,8,96,0,8,32,0,9,160,0,8,0,0,8,128,0,8,64,0,9,224,80,7,6,0,8,88,0,8,24,0,9,144,83,7,59,0,8,120,0,8,56,0,9,208,81,7,17,0,8,104,0,8,40,0,9,176,0,8,8,0,8,136,0,8,72,0,9,240,80,7,4,0,8,84,0,8,20,85,8,227,83,7,43,0,8,116,0,8,52,0,9,200,81,7,13,0,8,100,0,8,36,0,9,168,0,8,4,0,8,132,0,8,68,0,9,232,80,7,8,0,8,92,0,8,28,0,9,152,84,7,83,0,8,124,0,8,60,0,9,216,82,7,23,0,8,108,0,8,44,0,9,184,0,8,12,0,8,140,0,8,76,0,9,248,80,7,3,0,8,82,0,8,18,85,8,163,83,7,35,0,8,114,0,8,50,0,9,196,81,7,11,0,8,98,0,8,34,0,9,164,0,8,2,0,8,130,0,8,66,0,9,228,80,7,7,0,8,90,0,8,26,0,9,148,84,7,67,0,8,122,0,8,58,0,9,212,82,7,19,0,8,106,0,8,42,0,9,180,0,8,10,0,8,138,0,8,74,0,9,244,80,7,5,0,8,86,0,8,22,192,8,0,83,7,51,0,8,118,0,8,54,0,9,204,81,7,15,0,8,102,0,8,38,0,9,172,0,8,6,0,8,134,0,8,70,0,9,236,80,7,9,0,8,94,0,8,30,0,9,156,84,7,99,0,8,126,0,8,62,0,9,220,82,7,27,0,8,110,0,8,46,0,9,188,0,8,14,0,8,142,0,8,78,0,9,252,96,7,256,0,8,81,0,8,17,85,8,131,82,7,31,0,8,113,0,8,49,0,9,194,80,7,10,0,8,97,0,8,33,0,9,162,0,8,1,0,8,129,0,8,65,0,9,226,80,7,6,0,8,89,0,8,25,0,9,146,83,7,59,0,8,121,0,8,57,0,9,210,81,7,17,0,8,105,0,8,41,0,9,178,0,8,9,0,8,137,0,8,73,0,9,242,80,7,4,0,8,85,0,8,21,80,8,258,83,7,43,0,8,117,0,8,53,0,9,202,81,7,13,0,8,101,0,8,37,0,9,170,0,8,5,0,8,133,0,8,69,0,9,234,80,7,8,0,8,93,0,8,29,0,9,154,84,7,83,0,8,125,0,8,61,0,9,218,82,7,23,0,8,109,0,8,45,0,9,186,0,8,13,0,8,141,0,8,77,0,9,250,80,7,3,0,8,83,0,8,19,85,8,195,83,7,35,0,8,115,0,8,51,0,9,198,81,7,11,0,8,99,0,8,35,0,9,166,0,8,3,0,8,131,0,8,67,0,9,230,80,7,7,0,8,91,0,8,27,0,9,150,84,7,67,0,8,123,0,8,59,0,9,214,82,7,19,0,8,107,0,8,43,0,9,182,0,8,11,0,8,139,0,8,75,0,9,246,80,7,5,0,8,87,0,8,23,192,8,0,83,7,51,0,8,119,0,8,55,0,9,206,81,7,15,0,8,103,0,8,39,0,9,174,0,8,7,0,8,135,0,8,71,0,9,238,80,7,9,0,8,95,0,8,31,0,9,158,84,7,99,0,8,127,0,8,63,0,9,222,82,7,27,0,8,111,0,8,47,0,9,190,0,8,15,0,8,143,0,8,79,0,9,254,96,7,256,0,8,80,0,8,16,84,8,115,82,7,31,0,8,112,0,8,48,0,9,193,80,7,10,0,8,96,0,8,32,0,9,161,0,8,0,0,8,128,0,8,64,0,9,225,80,7,6,0,8,88,0,8,24,0,9,145,83,7,59,0,8,120,0,8,56,0,9,209,81,7,17,0,8,104,0,8,40,0,9,177,0,8,8,0,8,136,0,8,72,0,9,241,80,7,4,0,8,84,0,8,20,85,8,227,83,7,43,0,8,116,0,8,52,0,9,201,81,7,13,0,8,100,0,8,36,0,9,169,0,8,4,0,8,132,0,8,68,0,9,233,80,7,8,0,8,92,0,8,28,0,9,153,84,7,83,0,8,124,0,8,60,0,9,217,82,7,23,0,8,108,0,8,44,0,9,185,0,8,12,0,8,140,0,8,76,0,9,249,80,7,3,0,8,82,0,8,18,85,8,163,83,7,35,0,8,114,0,8,50,0,9,197,81,7,11,0,8,98,0,8,34,0,9,165,0,8,2,0,8,130,0,8,66,0,9,229,80,7,7,0,8,90,0,8,26,0,9,149,84,7,67,0,8,122,0,8,58,0,9,213,82,7,19,0,8,106,0,8,42,0,9,181,0,8,10,0,8,138,0,8,74,0,9,245,80,7,5,0,8,86,0,8,22,192,8,0,83,7,51,0,8,118,0,8,54,0,9,205,81,7,15,0,8,102,0,8,38,0,9,173,0,8,6,0,8,134,0,8,70,0,9,237,80,7,9,0,8,94,0,8,30,0,9,157,84,7,99,0,8,126,0,8,62,0,9,221,82,7,27,0,8,110,0,8,46,0,9,189,0,8,14,0,8,142,0,8,78,0,9,253,96,7,256,0,8,81,0,8,17,85,8,131,82,7,31,0,8,113,0,8,49,0,9,195,80,7,10,0,8,97,0,8,33,0,9,163,0,8,1,0,8,129,0,8,65,0,9,227,80,7,6,0,8,89,0,8,25,0,9,147,83,7,59,0,8,121,0,8,57,0,9,211,81,7,17,0,8,105,0,8,41,0,9,179,0,8,9,0,8,137,0,8,73,0,9,243,80,7,4,0,8,85,0,8,21,80,8,258,83,7,43,0,8,117,0,8,53,0,9,203,81,7,13,0,8,101,0,8,37,0,9,171,0,8,5,0,8,133,0,8,69,0,9,235,80,7,8,0,8,93,0,8,29,0,9,155,84,7,83,0,8,125,0,8,61,0,9,219,82,7,23,0,8,109,0,8,45,0,9,187,0,8,13,0,8,141,0,8,77,0,9,251,80,7,3,0,8,83,0,8,19,85,8,195,83,7,35,0,8,115,0,8,51,0,9,199,81,7,11,0,8,99,0,8,35,0,9,167,0,8,3,0,8,131,0,8,67,0,9,231,80,7,7,0,8,91,0,8,27,0,9,151,84,7,67,0,8,123,0,8,59,0,9,215,82,7,19,0,8,107,0,8,43,0,9,183,0,8,11,0,8,139,0,8,75,0,9,247,80,7,5,0,8,87,0,8,23,192,8,0,83,7,51,0,8,119,0,8,55,0,9,207,81,7,15,0,8,103,0,8,39,0,9,175,0,8,7,0,8,135,0,8,71,0,9,239,80,7,9,0,8,95,0,8,31,0,9,159,84,7,99,0,8,127,0,8,63,0,9,223,82,7,27,0,8,111,0,8,47,0,9,191,0,8,15,0,8,143,0,8,79,0,9,255],dt=[80,5,1,87,5,257,83,5,17,91,5,4097,81,5,5,89,5,1025,85,5,65,93,5,16385,80,5,3,88,5,513,84,5,33,92,5,8193,82,5,9,90,5,2049,86,5,129,192,5,24577,80,5,2,87,5,385,83,5,25,91,5,6145,81,5,7,89,5,1537,85,5,97,93,5,24577,80,5,4,88,5,769,84,5,49,92,5,12289,82,5,13,90,5,3073,86,5,193,192,5,24577],pt=[3,4,5,6,7,8,9,10,11,13,15,17,19,23,27,31,35,43,51,59,67,83,99,115,131,163,195,227,258,0,0],yt=[0,0,0,0,0,0,0,0,1,1,1,1,2,2,2,2,3,3,3,3,4,4,4,4,5,5,5,5,0,112,112],mt=[1,2,3,4,5,7,9,13,17,25,33,49,65,97,129,193,257,385,513,769,1025,1537,2049,3073,4097,6145,8193,12289,16385,24577],bt=[0,0,0,0,1,1,2,2,3,3,4,4,5,5,6,6,7,7,8,8,9,9,10,10,11,11,12,12,13,13],gt=15;function kt(){let e,t,n,r,s,i;function o(e,t,o,c,f,a,l,u,w,h,d){let p,y,m,b,g,k,v,S,z,C,x,A,_,I,P;C=0,g=o;do{n[e[t+C]]++,C++,g--}while(0!==g);if(n[0]==o)return l[0]=-1,u[0]=0,it;for(S=u[0],k=1;gt>=k&&0===n[k];k++);for(v=k,k>S&&(S=k),g=gt;0!==g&&0===n[g];g--);for(m=g,S>g&&(S=g),u[0]=S,I=1<<k;g>k;k++,I<<=1)if(0>(I-=n[k]))return ft;if(0>(I-=n[g]))return ft;for(n[g]+=I,i[1]=k=0,C=1,_=2;0!=--g;)i[_]=k+=n[C],_++,C++;g=0,C=0;do{0!==(k=e[t+C])&&(d[i[k]++]=g),C++}while(++g<o);for(o=i[m],i[0]=g=0,C=0,b=-1,A=-S,s[0]=0,x=0,P=0;m>=v;v++)for(p=n[v];0!=p--;){for(;v>A+S;){if(b++,A+=S,P=m-A,P=P>S?S:P,(y=1<<(k=v-A))>p+1&&(y-=p+1,_=v,P>k))for(;++k<P&&(y<<=1)>n[++_];)y-=n[_];if(P=1<<k,h[0]+P>wt)return ft;s[b]=x=h[0],h[0]+=P,0!==b?(i[b]=g,r[0]=k,r[1]=S,k=g>>>A-S,r[2]=x-s[b-1]-k,w.set(r,3*(s[b-1]+k))):l[0]=x}for(r[1]=v-A,o>C?d[C]<c?(r[0]=256>d[C]?0:96,r[2]=d[C++]):(r[0]=a[d[C]-c]+16+64,r[2]=f[d[C++]-c]):r[0]=192,y=1<<v-A,k=g>>>A;P>k;k+=y)w.set(r,3*(x+k));for(k=1<<v-1;g&k;k>>>=1)g^=k;for(g^=k,z=(1<<A)-1;(g&z)!=i[b];)b--,A-=S,z=(1<<A)-1}return 0!==I&&1!=m?lt:it}function c(o){let c;for(e||(e=[],t=[],n=new f(gt+1),r=[],s=new f(gt),i=new f(gt+1)),t.length<o&&(t=[]),c=0;o>c;c++)t[c]=0;for(c=0;gt+1>c;c++)n[c]=0;for(c=0;3>c;c++)r[c]=0;s.set(n.subarray(0,gt),0),i.set(n.subarray(0,gt+1),0)}this.it=(n,r,s,i,f)=>{let a;return c(19),e[0]=0,a=o(n,0,19,19,null,null,s,r,i,e,t),a==ft?f.Fe="oversubscribed dynamic bit lengths tree":a!=lt&&0!==r[0]||(f.Fe="incomplete dynamic bit lengths tree",a=ft),a},this.ot=(n,r,s,i,f,a,l,u,w)=>{let h;return c(288),e[0]=0,h=o(s,0,n,257,pt,yt,a,i,u,e,t),h!=it||0===i[0]?(h==ft?w.Fe="oversubscribed literal/length tree":h!=at&&(w.Fe="incomplete literal/length tree",h=ft),h):(c(288),h=o(s,n,r,0,mt,bt,l,f,u,e,t),h!=it||0===f[0]&&n>257?(h==ft?w.Fe="oversubscribed distance tree":h==lt?(w.Fe="incomplete distance tree",h=ft):h!=at&&(w.Fe="empty distance tree with lengths",h=ft),h):it)}}kt.ct=(e,t,n,r)=>(e[0]=9,t[0]=5,n[0]=ht,r[0]=dt,it);const vt=0,St=1,zt=2,Ct=3,xt=4,At=5,_t=6,It=7,Pt=8,Dt=9;function Vt(){const e=this;let t,n,r,s,i=0,o=0,c=0,f=0,a=0,l=0,u=0,w=0,h=0,d=0;function p(e,t,n,r,s,i,o,c){let f,a,l,u,w,h,d,p,y,m,b,g,k,v,S,z;d=c.rt,p=c.je,w=o.ft,h=o.lt,y=o.write,m=y<o.read?o.read-y-1:o.end-y,b=ut[e],g=ut[t];do{for(;20>h;)p--,w|=(255&c.ut(d++))<<h,h+=8;if(f=w&b,a=n,l=r,z=3*(l+f),0!==(u=a[z]))for(;;){if(w>>=a[z+1],h-=a[z+1],16&u){for(u&=15,k=a[z+2]+(w&ut[u]),w>>=u,h-=u;15>h;)p--,w|=(255&c.ut(d++))<<h,h+=8;for(f=w&g,a=s,l=i,z=3*(l+f),u=a[z];;){if(w>>=a[z+1],h-=a[z+1],16&u){for(u&=15;u>h;)p--,w|=(255&c.ut(d++))<<h,h+=8;if(v=a[z+2]+(w&ut[u]),w>>=u,h-=u,m-=k,v>y){S=y-v;do{S+=o.end}while(0>S);if(u=o.end-S,k>u){if(k-=u,y-S>0&&u>y-S)do{o.wt[y++]=o.wt[S++]}while(0!=--u);else o.wt.set(o.wt.subarray(S,S+u),y),y+=u,S+=u,u=0;S=0}}else S=y-v,y-S>0&&2>y-S?(o.wt[y++]=o.wt[S++],o.wt[y++]=o.wt[S++],k-=2):(o.wt.set(o.wt.subarray(S,S+2),y),y+=2,S+=2,k-=2);if(y-S>0&&k>y-S)do{o.wt[y++]=o.wt[S++]}while(0!=--k);else o.wt.set(o.wt.subarray(S,S+k),y),y+=k,S+=k,k=0;break}if(64&u)return c.Fe="invalid distance code",k=c.je-p,k=k>h>>3?h>>3:k,p+=k,d-=k,h-=k<<3,o.ft=w,o.lt=h,c.je=p,c.Ge+=d-c.rt,c.rt=d,o.write=y,ft;f+=a[z+2],f+=w&ut[u],z=3*(l+f),u=a[z]}break}if(64&u)return 32&u?(k=c.je-p,k=k>h>>3?h>>3:k,p+=k,d-=k,h-=k<<3,o.ft=w,o.lt=h,c.je=p,c.Ge+=d-c.rt,c.rt=d,o.write=y,ot):(c.Fe="invalid literal/length code",k=c.je-p,k=k>h>>3?h>>3:k,p+=k,d-=k,h-=k<<3,o.ft=w,o.lt=h,c.je=p,c.Ge+=d-c.rt,c.rt=d,o.write=y,ft);if(f+=a[z+2],f+=w&ut[u],z=3*(l+f),0===(u=a[z])){w>>=a[z+1],h-=a[z+1],o.wt[y++]=a[z+2],m--;break}}else w>>=a[z+1],h-=a[z+1],o.wt[y++]=a[z+2],m--}while(m>=258&&p>=10);return k=c.je-p,k=k>h>>3?h>>3:k,p+=k,d-=k,h-=k<<3,o.ft=w,o.lt=h,c.je=p,c.Ge+=d-c.rt,c.rt=d,o.write=y,it}e.init=(e,i,o,c,f,a)=>{t=vt,u=e,w=i,r=o,h=c,s=f,d=a,n=null},e.ht=(e,y,m)=>{let b,g,k,v,S,z,C,x=0,A=0,_=0;for(_=y.rt,v=y.je,x=e.ft,A=e.lt,S=e.write,z=S<e.read?e.read-S-1:e.end-S;;)switch(t){case vt:if(z>=258&&v>=10&&(e.ft=x,e.lt=A,y.je=v,y.Ge+=_-y.rt,y.rt=_,e.write=S,m=p(u,w,r,h,s,d,e,y),_=y.rt,v=y.je,x=e.ft,A=e.lt,S=e.write,z=S<e.read?e.read-S-1:e.end-S,m!=it)){t=m==ot?It:Dt;break}c=u,n=r,o=h,t=St;case St:for(b=c;b>A;){if(0===v)return e.ft=x,e.lt=A,y.je=v,y.Ge+=_-y.rt,y.rt=_,e.write=S,e.dt(y,m);m=it,v--,x|=(255&y.ut(_++))<<A,A+=8}if(g=3*(o+(x&ut[b])),x>>>=n[g+1],A-=n[g+1],k=n[g],0===k){f=n[g+2],t=_t;break}if(16&k){a=15&k,i=n[g+2],t=zt;break}if(!(64&k)){c=k,o=g/3+n[g+2];break}if(32&k){t=It;break}return t=Dt,y.Fe="invalid literal/length code",m=ft,e.ft=x,e.lt=A,y.je=v,y.Ge+=_-y.rt,y.rt=_,e.write=S,e.dt(y,m);case zt:for(b=a;b>A;){if(0===v)return e.ft=x,e.lt=A,y.je=v,y.Ge+=_-y.rt,y.rt=_,e.write=S,e.dt(y,m);m=it,v--,x|=(255&y.ut(_++))<<A,A+=8}i+=x&ut[b],x>>=b,A-=b,c=w,n=s,o=d,t=Ct;case Ct:for(b=c;b>A;){if(0===v)return e.ft=x,e.lt=A,y.je=v,y.Ge+=_-y.rt,y.rt=_,e.write=S,e.dt(y,m);m=it,v--,x|=(255&y.ut(_++))<<A,A+=8}if(g=3*(o+(x&ut[b])),x>>=n[g+1],A-=n[g+1],k=n[g],16&k){a=15&k,l=n[g+2],t=xt;break}if(!(64&k)){c=k,o=g/3+n[g+2];break}return t=Dt,y.Fe="invalid distance code",m=ft,e.ft=x,e.lt=A,y.je=v,y.Ge+=_-y.rt,y.rt=_,e.write=S,e.dt(y,m);case xt:for(b=a;b>A;){if(0===v)return e.ft=x,e.lt=A,y.je=v,y.Ge+=_-y.rt,y.rt=_,e.write=S,e.dt(y,m);m=it,v--,x|=(255&y.ut(_++))<<A,A+=8}l+=x&ut[b],x>>=b,A-=b,t=At;case At:for(C=S-l;0>C;)C+=e.end;for(;0!==i;){if(0===z&&(S==e.end&&0!==e.read&&(S=0,z=S<e.read?e.read-S-1:e.end-S),0===z&&(e.write=S,m=e.dt(y,m),S=e.write,z=S<e.read?e.read-S-1:e.end-S,S==e.end&&0!==e.read&&(S=0,z=S<e.read?e.read-S-1:e.end-S),0===z)))return e.ft=x,e.lt=A,y.je=v,y.Ge+=_-y.rt,y.rt=_,e.write=S,e.dt(y,m);e.wt[S++]=e.wt[C++],z--,C==e.end&&(C=0),i--}t=vt;break;case _t:if(0===z&&(S==e.end&&0!==e.read&&(S=0,z=S<e.read?e.read-S-1:e.end-S),0===z&&(e.write=S,m=e.dt(y,m),S=e.write,z=S<e.read?e.read-S-1:e.end-S,S==e.end&&0!==e.read&&(S=0,z=S<e.read?e.read-S-1:e.end-S),0===z)))return e.ft=x,e.lt=A,y.je=v,y.Ge+=_-y.rt,y.rt=_,e.write=S,e.dt(y,m);m=it,e.wt[S++]=f,z--,t=vt;break;case It:if(A>7&&(A-=8,v++,_--),e.write=S,m=e.dt(y,m),S=e.write,z=S<e.read?e.read-S-1:e.end-S,e.read!=e.write)return e.ft=x,e.lt=A,y.je=v,y.Ge+=_-y.rt,y.rt=_,e.write=S,e.dt(y,m);t=Pt;case Pt:return m=ot,e.ft=x,e.lt=A,y.je=v,y.Ge+=_-y.rt,y.rt=_,e.write=S,e.dt(y,m);case Dt:return m=ft,e.ft=x,e.lt=A,y.je=v,y.Ge+=_-y.rt,y.rt=_,e.write=S,e.dt(y,m);default:return m=ct,e.ft=x,e.lt=A,y.je=v,y.Ge+=_-y.rt,y.rt=_,e.write=S,e.dt(y,m)}},e.yt=()=>{}}const Rt=[16,17,18,0,8,7,9,6,10,5,11,4,12,3,13,2,14,1,15],Bt=0,Et=1,Mt=2,Ut=3,Kt=4,Nt=5,Ot=6,Tt=7,Wt=8,jt=9;function Ht(e,t){const n=this;let r,s=Bt,o=0,c=0,a=0;const l=[0],u=[0],w=new Vt;let h=0,d=new f(3*wt);const p=new kt;n.lt=0,n.ft=0,n.wt=new i(t),n.end=t,n.read=0,n.write=0,n.reset=(e,t)=>{t&&(t[0]=0),s==Ot&&w.yt(e),s=Bt,n.lt=0,n.ft=0,n.read=n.write=0},n.reset(e,null),n.dt=(e,t)=>{let r,s,i;return s=e.st,i=n.read,r=(i>n.write?n.end:n.write)-i,r>e.nt&&(r=e.nt),0!==r&&t==lt&&(t=it),e.nt-=r,e.Je+=r,e.et.set(n.wt.subarray(i,i+r),s),s+=r,i+=r,i==n.end&&(i=0,n.write==n.end&&(n.write=0),r=n.write-i,r>e.nt&&(r=e.nt),0!==r&&t==lt&&(t=it),e.nt-=r,e.Je+=r,e.et.set(n.wt.subarray(i,i+r),s),s+=r,i+=r),e.st=s,n.read=i,t},n.ht=(e,t)=>{let i,f,y,m,b,g,k,v;for(m=e.rt,b=e.je,f=n.ft,y=n.lt,g=n.write,k=g<n.read?n.read-g-1:n.end-g;;){let S,z,C,x,A,_,I,P;switch(s){case Bt:for(;3>y;){if(0===b)return n.ft=f,n.lt=y,e.je=b,e.Ge+=m-e.rt,e.rt=m,n.write=g,n.dt(e,t);t=it,b--,f|=(255&e.ut(m++))<<y,y+=8}switch(i=7&f,h=1&i,i>>>1){case 0:f>>>=3,y-=3,i=7&y,f>>>=i,y-=i,s=Et;break;case 1:S=[],z=[],C=[[]],x=[[]],kt.ct(S,z,C,x),w.init(S[0],z[0],C[0],0,x[0],0),f>>>=3,y-=3,s=Ot;break;case 2:f>>>=3,y-=3,s=Ut;break;case 3:return f>>>=3,y-=3,s=jt,e.Fe="invalid block type",t=ft,n.ft=f,n.lt=y,e.je=b,e.Ge+=m-e.rt,e.rt=m,n.write=g,n.dt(e,t)}break;case Et:for(;32>y;){if(0===b)return n.ft=f,n.lt=y,e.je=b,e.Ge+=m-e.rt,e.rt=m,n.write=g,n.dt(e,t);t=it,b--,f|=(255&e.ut(m++))<<y,y+=8}if((~f>>>16&65535)!=(65535&f))return s=jt,e.Fe="invalid stored block lengths",t=ft,n.ft=f,n.lt=y,e.je=b,e.Ge+=m-e.rt,e.rt=m,n.write=g,n.dt(e,t);o=65535&f,f=y=0,s=0!==o?Mt:0!==h?Tt:Bt;break;case Mt:if(0===b)return n.ft=f,n.lt=y,e.je=b,e.Ge+=m-e.rt,e.rt=m,n.write=g,n.dt(e,t);if(0===k&&(g==n.end&&0!==n.read&&(g=0,k=g<n.read?n.read-g-1:n.end-g),0===k&&(n.write=g,t=n.dt(e,t),g=n.write,k=g<n.read?n.read-g-1:n.end-g,g==n.end&&0!==n.read&&(g=0,k=g<n.read?n.read-g-1:n.end-g),0===k)))return n.ft=f,n.lt=y,e.je=b,e.Ge+=m-e.rt,e.rt=m,n.write=g,n.dt(e,t);if(t=it,i=o,i>b&&(i=b),i>k&&(i=k),n.wt.set(e.He(m,i),g),m+=i,b-=i,g+=i,k-=i,0!=(o-=i))break;s=0!==h?Tt:Bt;break;case Ut:for(;14>y;){if(0===b)return n.ft=f,n.lt=y,e.je=b,e.Ge+=m-e.rt,e.rt=m,n.write=g,n.dt(e,t);t=it,b--,f|=(255&e.ut(m++))<<y,y+=8}if(c=i=16383&f,(31&i)>29||(i>>5&31)>29)return s=jt,e.Fe="too many length or distance symbols",t=ft,n.ft=f,n.lt=y,e.je=b,e.Ge+=m-e.rt,e.rt=m,n.write=g,n.dt(e,t);if(i=258+(31&i)+(i>>5&31),!r||r.length<i)r=[];else for(v=0;i>v;v++)r[v]=0;f>>>=14,y-=14,a=0,s=Kt;case Kt:for(;4+(c>>>10)>a;){for(;3>y;){if(0===b)return n.ft=f,n.lt=y,e.je=b,e.Ge+=m-e.rt,e.rt=m,n.write=g,n.dt(e,t);t=it,b--,f|=(255&e.ut(m++))<<y,y+=8}r[Rt[a++]]=7&f,f>>>=3,y-=3}for(;19>a;)r[Rt[a++]]=0;if(l[0]=7,i=p.it(r,l,u,d,e),i!=it)return(t=i)==ft&&(r=null,s=jt),n.ft=f,n.lt=y,e.je=b,e.Ge+=m-e.rt,e.rt=m,n.write=g,n.dt(e,t);a=0,s=Nt;case Nt:for(;i=c,258+(31&i)+(i>>5&31)>a;){let o,w;for(i=l[0];i>y;){if(0===b)return n.ft=f,n.lt=y,e.je=b,e.Ge+=m-e.rt,e.rt=m,n.write=g,n.dt(e,t);t=it,b--,f|=(255&e.ut(m++))<<y,y+=8}if(i=d[3*(u[0]+(f&ut[i]))+1],w=d[3*(u[0]+(f&ut[i]))+2],16>w)f>>>=i,y-=i,r[a++]=w;else{for(v=18==w?7:w-14,o=18==w?11:3;i+v>y;){if(0===b)return n.ft=f,n.lt=y,e.je=b,e.Ge+=m-e.rt,e.rt=m,n.write=g,n.dt(e,t);t=it,b--,f|=(255&e.ut(m++))<<y,y+=8}if(f>>>=i,y-=i,o+=f&ut[v],f>>>=v,y-=v,v=a,i=c,v+o>258+(31&i)+(i>>5&31)||16==w&&1>v)return r=null,s=jt,e.Fe="invalid bit length repeat",t=ft,n.ft=f,n.lt=y,e.je=b,e.Ge+=m-e.rt,e.rt=m,n.write=g,n.dt(e,t);w=16==w?r[v-1]:0;do{r[v++]=w}while(0!=--o);a=v}}if(u[0]=-1,A=[],_=[],I=[],P=[],A[0]=9,_[0]=6,i=c,i=p.ot(257+(31&i),1+(i>>5&31),r,A,_,I,P,d,e),i!=it)return i==ft&&(r=null,s=jt),t=i,n.ft=f,n.lt=y,e.je=b,e.Ge+=m-e.rt,e.rt=m,n.write=g,n.dt(e,t);w.init(A[0],_[0],d,I[0],d,P[0]),s=Ot;case Ot:if(n.ft=f,n.lt=y,e.je=b,e.Ge+=m-e.rt,e.rt=m,n.write=g,(t=w.ht(n,e,t))!=ot)return n.dt(e,t);if(t=it,w.yt(e),m=e.rt,b=e.je,f=n.ft,y=n.lt,g=n.write,k=g<n.read?n.read-g-1:n.end-g,0===h){s=Bt;break}s=Tt;case Tt:if(n.write=g,t=n.dt(e,t),g=n.write,k=g<n.read?n.read-g-1:n.end-g,n.read!=n.write)return n.ft=f,n.lt=y,e.je=b,e.Ge+=m-e.rt,e.rt=m,n.write=g,n.dt(e,t);s=Wt;case Wt:return t=ot,n.ft=f,n.lt=y,e.je=b,e.Ge+=m-e.rt,e.rt=m,n.write=g,n.dt(e,t);case jt:return t=ft,n.ft=f,n.lt=y,e.je=b,e.Ge+=m-e.rt,e.rt=m,n.write=g,n.dt(e,t);default:return t=ct,n.ft=f,n.lt=y,e.je=b,e.Ge+=m-e.rt,e.rt=m,n.write=g,n.dt(e,t)}}},n.yt=e=>{n.reset(e,null),n.wt=null,d=null},n.bt=(e,t,r)=>{n.wt.set(e.subarray(t,t+r),0),n.read=n.write=r},n.gt=()=>s==Et?1:0}const Lt=13,Ft=[0,0,255,255];function qt(){const e=this;function t(e){return e&&e.kt?(e.Ge=e.Je=0,e.Fe=null,e.kt.mode=7,e.kt.vt.reset(e,null),it):ct}e.mode=0,e.method=0,e.St=[0],e.zt=0,e.marker=0,e.Ct=0,e.xt=t=>(e.vt&&e.vt.yt(t),e.vt=null,it),e.At=(n,r)=>(n.Fe=null,e.vt=null,8>r||r>15?(e.xt(n),ct):(e.Ct=r,n.kt.vt=new Ht(n,1<<r),t(n),it)),e._t=(e,t)=>{let n,r;if(!e||!e.kt||!e.tt)return ct;const s=e.kt;for(t=4==t?lt:it,n=lt;;)switch(s.mode){case 0:if(0===e.je)return n;if(n=t,e.je--,e.Ge++,8!=(15&(s.method=e.ut(e.rt++)))){s.mode=Lt,e.Fe="unknown compression method",s.marker=5;break}if(8+(s.method>>4)>s.Ct){s.mode=Lt,e.Fe="invalid win size",s.marker=5;break}s.mode=1;case 1:if(0===e.je)return n;if(n=t,e.je--,e.Ge++,r=255&e.ut(e.rt++),((s.method<<8)+r)%31!=0){s.mode=Lt,e.Fe="incorrect header check",s.marker=5;break}if(!(32&r)){s.mode=7;break}s.mode=2;case 2:if(0===e.je)return n;n=t,e.je--,e.Ge++,s.zt=(255&e.ut(e.rt++))<<24&4278190080,s.mode=3;case 3:if(0===e.je)return n;n=t,e.je--,e.Ge++,s.zt+=(255&e.ut(e.rt++))<<16&16711680,s.mode=4;case 4:if(0===e.je)return n;n=t,e.je--,e.Ge++,s.zt+=(255&e.ut(e.rt++))<<8&65280,s.mode=5;case 5:return 0===e.je?n:(n=t,e.je--,e.Ge++,s.zt+=255&e.ut(e.rt++),s.mode=6,2);case 6:return s.mode=Lt,e.Fe="need dictionary",s.marker=0,ct;case 7:if(n=s.vt.ht(e,n),n==ft){s.mode=Lt,s.marker=0;break}if(n==it&&(n=t),n!=ot)return n;n=t,s.vt.reset(e,s.St),s.mode=12;case 12:return e.je=0,ot;case Lt:return ft;default:return ct}},e.It=(e,t,n)=>{let r=0,s=n;if(!e||!e.kt||6!=e.kt.mode)return ct;const i=e.kt;return s<1<<i.Ct||(s=(1<<i.Ct)-1,r=n-s),i.vt.bt(t,r,s),i.mode=7,it},e.Pt=e=>{let n,r,s,i,o;if(!e||!e.kt)return ct;const c=e.kt;if(c.mode!=Lt&&(c.mode=Lt,c.marker=0),0===(n=e.je))return lt;for(r=e.rt,s=c.marker;0!==n&&4>s;)e.ut(r)==Ft[s]?s++:s=0!==e.ut(r)?0:4-s,r++,n--;return e.Ge+=r-e.rt,e.rt=r,e.je=n,c.marker=s,4!=s?ft:(i=e.Ge,o=e.Je,t(e),e.Ge=i,e.Je=o,c.mode=7,it)},e.Dt=e=>e&&e.kt&&e.kt.vt?e.kt.vt.gt():ct}function Gt(){}function Jt(e){const t=new Gt,n=e&&e.chunkSize?r.floor(2*e.chunkSize):131072,o=new i(n);let c=!1;t.At(),t.et=o,this.append=(e,r)=>{const f=[];let a,l,u=0,w=0,h=0;if(0!==e.length){t.rt=0,t.tt=e,t.je=e.length;do{if(t.st=0,t.nt=n,0!==t.je||c||(t.rt=0,c=!0),a=t._t(0),c&&a===lt){if(0!==t.je)throw new s("inflating: bad input")}else if(a!==it&&a!==ot)throw new s("inflating: "+t.Fe);if((c||a===ot)&&t.je===e.length)throw new s("inflating: bad input");t.st&&(t.st===n?f.push(new i(o)):f.push(o.subarray(0,t.st))),h+=t.st,r&&t.rt>0&&t.rt!=u&&(r(t.rt),u=t.rt)}while(t.je>0||0===t.nt);return f.length>1?(l=new i(h),f.forEach((e=>{l.set(e,w),w+=e.length}))):l=f[0]?new i(f[0]):new i,l}},this.flush=()=>{t.xt()}}Gt.prototype={At(e){const t=this;return t.kt=new qt,e||(e=15),t.kt.At(t,e)},_t(e){const t=this;return t.kt?t.kt._t(t,e):ct},xt(){const e=this;if(!e.kt)return ct;const t=e.kt.xt(e);return e.kt=null,t},Pt(){const e=this;return e.kt?e.kt.Pt(e):ct},It(e,t){const n=this;return n.kt?n.kt.It(n,e,t):ct},ut(e){return this.tt[e]},He(e,t){return this.tt.subarray(e,e+t)}},self.initCodec=()=>{self.Deflate=st,self.Inflate=Jt};\n' ], {
        type: "text/javascript"
    }));
    e({
        workerScripts: {
            inflate: [ t ],
            deflate: [ t ]
        }
    });
}(Ue), Ue({
    Deflate: function(e) {
        const t = new q, r = (n = e && e.chunkSize ? e.chunkSize : 65536) + 5 * (Math.floor(n / 16383) + 1);
        var n;
        const o = new Uint8Array(r);
        let i = e ? e.level : -1;
        void 0 === i && (i = -1), t.deflateInit(i), t.next_out = o, this.append = function(e, n) {
            let i, a, s = 0, u = 0, l = 0;
            const c = [];
            if (e.length) {
                t.next_in_index = 0, t.next_in = e, t.avail_in = e.length;
                do {
                    if (t.next_out_index = 0, t.avail_out = r, i = t.deflate(0), 0 != i) {
                        throw new Error("deflating: " + t.msg);
                    }
                    t.next_out_index && (t.next_out_index == r ? c.push(new Uint8Array(o)) : c.push(o.subarray(0, t.next_out_index))), 
                    l += t.next_out_index, n && t.next_in_index > 0 && t.next_in_index != s && (n(t.next_in_index), 
                    s = t.next_in_index);
                } while (t.avail_in > 0 || 0 === t.avail_out);
                return c.length > 1 ? (a = new Uint8Array(l), c.forEach(function(e) {
                    a.set(e, u), u += e.length;
                })) : a = c[0] ? new Uint8Array(c[0]) : new Uint8Array, a;
            }
        }, this.flush = function() {
            let e, n, i = 0, a = 0;
            const s = [];
            do {
                if (t.next_out_index = 0, t.avail_out = r, e = t.deflate(4), 1 != e && 0 != e) {
                    throw new Error("deflating: " + t.msg);
                }
                r - t.avail_out > 0 && s.push(o.slice(0, t.next_out_index)), a += t.next_out_index;
            } while (t.avail_in > 0 || 0 === t.avail_out);
            return t.deflateEnd(), n = new Uint8Array(a), s.forEach(function(e) {
                n.set(e, i), i += e.length;
            }), n;
        };
    },
    Inflate: function(e) {
        const t = new pe, r = e && e.chunkSize ? Math.floor(2 * e.chunkSize) : 131072, n = new Uint8Array(r);
        let o = !1;
        t.inflateInit(), t.next_out = n, this.append = function(e, i) {
            const a = [];
            let s, u, l = 0, c = 0, f = 0;
            if (0 !== e.length) {
                t.next_in_index = 0, t.next_in = e, t.avail_in = e.length;
                do {
                    if (t.next_out_index = 0, t.avail_out = r, 0 !== t.avail_in || o || (t.next_in_index = 0, 
                    o = !0), s = t.inflate(0), o && s === Z) {
                        if (0 !== t.avail_in) {
                            throw new Error("inflating: bad input");
                        }
                    } else if (0 !== s && 1 !== s) {
                        throw new Error("inflating: " + t.msg);
                    }
                    if ((o || 1 === s) && t.avail_in === e.length) {
                        throw new Error("inflating: bad input");
                    }
                    t.next_out_index && (t.next_out_index === r ? a.push(new Uint8Array(n)) : a.push(n.subarray(0, t.next_out_index))), 
                    f += t.next_out_index, i && t.next_in_index > 0 && t.next_in_index != l && (i(t.next_in_index), 
                    l = t.next_in_index);
                } while (t.avail_in > 0 || 0 === t.avail_out);
                return a.length > 1 ? (u = new Uint8Array(f), a.forEach(function(e) {
                    u.set(e, c), c += e.length;
                })) : u = a[0] ? new Uint8Array(a[0]) : new Uint8Array, u;
            }
        }, this.flush = function() {
            t.inflateEnd();
        };
    }
}), O.BlobReader = kr, O.BlobWriter = Rr, O.Data64URIReader = Pr, O.Data64URIWriter = Ir, 
O.ERR_BAD_FORMAT = Pn, O.ERR_CENTRAL_DIRECTORY_NOT_FOUND = Rn, O.ERR_DUPLICATED_NAME = oo, 
O.ERR_ENCRYPTED = Ln, O.ERR_EOCDR_LOCATOR_ZIP64_NOT_FOUND = kn, O.ERR_EOCDR_NOT_FOUND = In, 
O.ERR_EXTRAFIELD_ZIP64_NOT_FOUND = jn, O.ERR_HTTP_RANGE = br, O.ERR_INVALID_COMMENT = io, 
O.ERR_INVALID_ENCRYPTION_STRENGTH = lo, O.ERR_INVALID_ENTRY_COMMENT = ao, O.ERR_INVALID_ENTRY_NAME = so, 
O.ERR_INVALID_EXTRAFIELD_DATA = fo, O.ERR_INVALID_EXTRAFIELD_TYPE = co, O.ERR_INVALID_PASSWORD = rt, 
O.ERR_INVALID_SIGNATURE = nt, O.ERR_INVALID_VERSION = uo, O.ERR_ITERATOR_COMPLETED_TOO_SOON = wr, 
O.ERR_LOCAL_FILE_HEADER_NOT_FOUND = Tn, O.ERR_SPLIT_ZIP_FILE = Un, O.ERR_UNSUPPORTED_COMPRESSION = Bn, 
O.ERR_UNSUPPORTED_ENCRYPTION = Nn, O.ERR_UNSUPPORTED_FORMAT = po, O.HttpRangeReader = class extends Xr {
    constructor(e, t = {}) {
        t.useRangeHeader = !0, super(e, t);
    }
}, O.HttpReader = Xr, O.Reader = Fr, O.SplitDataReader = Qr, O.SplitDataWriter = en, 
O.SplitZipReader = an, O.SplitZipWriter = sn, O.TextReader = Tr, O.TextWriter = jr, 
O.Uint8ArrayReader = Zr, O.Uint8ArrayWriter = Yr, O.Writer = Mr, O.ZipReader = Wn, 
O.ZipReaderStream = class {
    constructor(e = {}) {
        const {readable: t, writable: r} = new TransformStream, n = new Wn(t, e).getEntriesGenerator();
        this.readable = new ReadableStream({
            async pull(e) {
                const {done: t, value: r} = await n.next();
                if (t) {
                    return e.close();
                }
                const o = {
                    ...r,
                    readable: function() {
                        const {readable: e, writable: t} = new TransformStream;
                        if (r.getData) {
                            return r.getData(t), e;
                        }
                    }()
                };
                delete o.getData, e.enqueue(o);
            }
        }), this.writable = r;
    }
}, O.ZipWriter = mo, O.ZipWriterStream = class {
    constructor(e = {}) {
        const {readable: t, writable: r} = new TransformStream;
        this.readable = t, this.zipWriter = new mo(r, e);
    }
    transform(e) {
        const {readable: t, writable: r} = new TransformStream({
            flush: () => {
                this.zipWriter.close();
            }
        });
        return this.zipWriter.add(e, t), {
            readable: this.readable,
            writable: r
        };
    }
    writable(e) {
        const {readable: t, writable: r} = new TransformStream;
        return this.zipWriter.add(e, t), r;
    }
    close(e = void 0, t = {}) {
        return this.zipWriter.close(e, t);
    }
}, O.configure = Ue, O.fs = Po, O.getMimeType = function(e) {
    return e && $e[e.split(".").pop().toLowerCase()] || "application/octet-stream";
}, O.initShimAsyncCodec = function(e, t = {}, r) {
    return {
        Deflate: _r(e.Deflate, t.deflate, r),
        Inflate: _r(e.Inflate, t.inflate, r)
    };
}, O.terminateWorkers = async function() {
    await Promise.allSettled(hr.map(e => (yr(e), e.terminate())));
};

var Uo = {}, zo = {
    fromCallback: function(e) {
        return Object.defineProperty(function(...t) {
            if ("function" != typeof t[t.length - 1]) {
                return new Promise((r, n) => {
                    t.push((e, t) => null != e ? n(e) : r(t)), e.apply(this, t);
                });
            }
            e.apply(this, t);
        }, "name", {
            value: e.name
        });
    },
    fromPromise: function(e) {
        return Object.defineProperty(function(...t) {
            const r = t[t.length - 1];
            if ("function" != typeof r) {
                return e.apply(this, t);
            }
            t.pop(), e.apply(this, t).then(e => r(null, e), r);
        }, "name", {
            value: e.name
        });
    }
}, Ho = s, $o = process.cwd, Go = null, Wo = process.env.GRACEFUL_FS_PLATFORM || process.platform;

process.cwd = function() {
    return Go || (Go = $o.call(process)), Go;
};

try {
    process.cwd();
} catch (e) {}

if ("function" == typeof process.chdir) {
    var Vo = process.chdir;
    process.chdir = function(e) {
        Go = null, Vo.call(process, e);
    }, Object.setPrototypeOf && Object.setPrototypeOf(process.chdir, Vo);
}

var Ko = function(e) {
    Ho.hasOwnProperty("O_SYMLINK") && process.version.match(/^v0\.6\.[0-2]|^v0\.5\./) && function(e) {
        e.lchmod = function(t, r, n) {
            e.open(t, Ho.O_WRONLY | Ho.O_SYMLINK, r, function(t, o) {
                t ? n && n(t) : e.fchmod(o, r, function(t) {
                    e.close(o, function(e) {
                        n && n(t || e);
                    });
                });
            });
        }, e.lchmodSync = function(t, r) {
            var n, o = e.openSync(t, Ho.O_WRONLY | Ho.O_SYMLINK, r), i = !0;
            try {
                n = e.fchmodSync(o, r), i = !1;
            } finally {
                if (i) {
                    try {
                        e.closeSync(o);
                    } catch (e) {}
                } else {
                    e.closeSync(o);
                }
            }
            return n;
        };
    }(e);
    e.lutimes || function(e) {
        Ho.hasOwnProperty("O_SYMLINK") && e.futimes ? (e.lutimes = function(t, r, n, o) {
            e.open(t, Ho.O_SYMLINK, function(t, i) {
                t ? o && o(t) : e.futimes(i, r, n, function(t) {
                    e.close(i, function(e) {
                        o && o(t || e);
                    });
                });
            });
        }, e.lutimesSync = function(t, r, n) {
            var o, i = e.openSync(t, Ho.O_SYMLINK), a = !0;
            try {
                o = e.futimesSync(i, r, n), a = !1;
            } finally {
                if (a) {
                    try {
                        e.closeSync(i);
                    } catch (e) {}
                } else {
                    e.closeSync(i);
                }
            }
            return o;
        }) : e.futimes && (e.lutimes = function(e, t, r, n) {
            n && process.nextTick(n);
        }, e.lutimesSync = function() {});
    }(e);
    e.chown = n(e.chown), e.fchown = n(e.fchown), e.lchown = n(e.lchown), e.chmod = t(e.chmod), 
    e.fchmod = t(e.fchmod), e.lchmod = t(e.lchmod), e.chownSync = o(e.chownSync), e.fchownSync = o(e.fchownSync), 
    e.lchownSync = o(e.lchownSync), e.chmodSync = r(e.chmodSync), e.fchmodSync = r(e.fchmodSync), 
    e.lchmodSync = r(e.lchmodSync), e.stat = i(e.stat), e.fstat = i(e.fstat), e.lstat = i(e.lstat), 
    e.statSync = a(e.statSync), e.fstatSync = a(e.fstatSync), e.lstatSync = a(e.lstatSync), 
    e.chmod && !e.lchmod && (e.lchmod = function(e, t, r) {
        r && process.nextTick(r);
    }, e.lchmodSync = function() {});
    e.chown && !e.lchown && (e.lchown = function(e, t, r, n) {
        n && process.nextTick(n);
    }, e.lchownSync = function() {});
    "win32" === Wo && (e.rename = "function" != typeof e.rename ? e.rename : function(t) {
        function r(r, n, o) {
            var i = Date.now(), a = 0;
            t(r, n, function s(u) {
                if (u && ("EACCES" === u.code || "EPERM" === u.code || "EBUSY" === u.code) && Date.now() - i < 6e4) {
                    return setTimeout(function() {
                        e.stat(n, function(e, i) {
                            e && "ENOENT" === e.code ? t(r, n, s) : o(u);
                        });
                    }, a), void (a < 100 && (a += 10));
                }
                o && o(u);
            });
        }
        return Object.setPrototypeOf && Object.setPrototypeOf(r, t), r;
    }(e.rename));
    function t(t) {
        return t ? function(r, n, o) {
            return t.call(e, r, n, function(e) {
                s(e) && (e = null), o && o.apply(this, arguments);
            });
        } : t;
    }
    function r(t) {
        return t ? function(r, n) {
            try {
                return t.call(e, r, n);
            } catch (e) {
                if (!s(e)) {
                    throw e;
                }
            }
        } : t;
    }
    function n(t) {
        return t ? function(r, n, o, i) {
            return t.call(e, r, n, o, function(e) {
                s(e) && (e = null), i && i.apply(this, arguments);
            });
        } : t;
    }
    function o(t) {
        return t ? function(r, n, o) {
            try {
                return t.call(e, r, n, o);
            } catch (e) {
                if (!s(e)) {
                    throw e;
                }
            }
        } : t;
    }
    function i(t) {
        return t ? function(r, n, o) {
            function i(e, t) {
                t && (t.uid < 0 && (t.uid += 4294967296), t.gid < 0 && (t.gid += 4294967296)), o && o.apply(this, arguments);
            }
            return "function" == typeof n && (o = n, n = null), n ? t.call(e, r, n, i) : t.call(e, r, i);
        } : t;
    }
    function a(t) {
        return t ? function(r, n) {
            var o = n ? t.call(e, r, n) : t.call(e, r);
            return o && (o.uid < 0 && (o.uid += 4294967296), o.gid < 0 && (o.gid += 4294967296)), 
            o;
        } : t;
    }
    function s(e) {
        return !e || ("ENOSYS" === e.code || !(process.getuid && 0 === process.getuid() || "EINVAL" !== e.code && "EPERM" !== e.code));
    }
    e.read = "function" != typeof e.read ? e.read : function(t) {
        function r(r, n, o, i, a, s) {
            var u;
            if (s && "function" == typeof s) {
                var l = 0;
                u = function(c, f, d) {
                    if (c && "EAGAIN" === c.code && l < 10) {
                        return l++, t.call(e, r, n, o, i, a, u);
                    }
                    s.apply(this, arguments);
                };
            }
            return t.call(e, r, n, o, i, a, u);
        }
        return Object.setPrototypeOf && Object.setPrototypeOf(r, t), r;
    }(e.read), e.readSync = "function" != typeof e.readSync ? e.readSync : (u = e.readSync, 
    function(t, r, n, o, i) {
        for (var a = 0; ;) {
            try {
                return u.call(e, t, r, n, o, i);
            } catch (e) {
                if ("EAGAIN" === e.code && a < 10) {
                    a++;
                    continue;
                }
                throw e;
            }
        }
    });
    var u;
};

var qo = u.Stream, Jo = function(e) {
    return {
        ReadStream: function t(r, n) {
            if (!(this instanceof t)) {
                return new t(r, n);
            }
            qo.call(this);
            var o = this;
            this.path = r, this.fd = null, this.readable = !0, this.paused = !1, this.flags = "r", 
            this.mode = 438, this.bufferSize = 65536, n = n || {};
            for (var i = Object.keys(n), a = 0, s = i.length; a < s; a++) {
                var u = i[a];
                this[u] = n[u];
            }
            this.encoding && this.setEncoding(this.encoding);
            if (void 0 !== this.start) {
                if ("number" != typeof this.start) {
                    throw TypeError("start must be a Number");
                }
                if (void 0 === this.end) {
                    this.end = 1 / 0;
                } else if ("number" != typeof this.end) {
                    throw TypeError("end must be a Number");
                }
                if (this.start > this.end) {
                    throw new Error("start must be <= end");
                }
                this.pos = this.start;
            }
            if (null !== this.fd) {
                return void process.nextTick(function() {
                    o._read();
                });
            }
            e.open(this.path, this.flags, this.mode, function(e, t) {
                if (e) {
                    return o.emit("error", e), void (o.readable = !1);
                }
                o.fd = t, o.emit("open", t), o._read();
            });
        },
        WriteStream: function t(r, n) {
            if (!(this instanceof t)) {
                return new t(r, n);
            }
            qo.call(this), this.path = r, this.fd = null, this.writable = !0, this.flags = "w", 
            this.encoding = "binary", this.mode = 438, this.bytesWritten = 0, n = n || {};
            for (var o = Object.keys(n), i = 0, a = o.length; i < a; i++) {
                var s = o[i];
                this[s] = n[s];
            }
            if (void 0 !== this.start) {
                if ("number" != typeof this.start) {
                    throw TypeError("start must be a Number");
                }
                if (this.start < 0) {
                    throw new Error("start must be >= zero");
                }
                this.pos = this.start;
            }
            this.busy = !1, this._queue = [], null === this.fd && (this._open = e.open, this._queue.push([ this._open, this.path, this.flags, this.mode, void 0 ]), 
            this.flush());
        }
    };
};

var Xo = function(e) {
    if (null === e || "object" != typeof e) {
        return e;
    }
    if (e instanceof Object) {
        var t = {
            __proto__: Zo(e)
        };
    } else {
        t = Object.create(null);
    }
    return Object.getOwnPropertyNames(e).forEach(function(r) {
        Object.defineProperty(t, r, Object.getOwnPropertyDescriptor(e, r));
    }), t;
}, Zo = Object.getPrototypeOf || function(e) {
    return e.__proto__;
};

var Yo, Qo, ei = t, ti = Ko, ri = Jo, ni = Xo, oi = l;

function ii(e, t) {
    Object.defineProperty(e, Yo, {
        get: function() {
            return t;
        }
    });
}

"function" == typeof Symbol && "function" == typeof Symbol.for ? (Yo = Symbol.for("graceful-fs.queue"), 
Qo = Symbol.for("graceful-fs.previous")) : (Yo = "___graceful-fs.queue", Qo = "___graceful-fs.previous");

var ai = function() {};

if (oi.debuglog ? ai = oi.debuglog("gfs4") : /\bgfs4\b/i.test(process.env.NODE_DEBUG || "") && (ai = function() {
    var e = oi.format.apply(oi, arguments);
    e = "GFS4: " + e.split(/\n/).join("\nGFS4: "), console.error(e);
}), !ei[Yo]) {
    var si = g[Yo] || [];
    ii(ei, si), ei.close = function(e) {
        function t(t, r) {
            return e.call(ei, t, function(e) {
                e || di(), "function" == typeof r && r.apply(this, arguments);
            });
        }
        return Object.defineProperty(t, Qo, {
            value: e
        }), t;
    }(ei.close), ei.closeSync = function(e) {
        function t(t) {
            e.apply(ei, arguments), di();
        }
        return Object.defineProperty(t, Qo, {
            value: e
        }), t;
    }(ei.closeSync), /\bgfs4\b/i.test(process.env.NODE_DEBUG || "") && process.on("exit", function() {
        ai(ei[Yo]), c.equal(ei[Yo].length, 0);
    });
}

g[Yo] || ii(g, ei[Yo]);

var ui, li = ci(ni(ei));

function ci(e) {
    ti(e), e.gracefulify = ci, e.createReadStream = function(t, r) {
        return new e.ReadStream(t, r);
    }, e.createWriteStream = function(t, r) {
        return new e.WriteStream(t, r);
    };
    var t = e.readFile;
    e.readFile = function(e, r, n) {
        "function" == typeof r && (n = r, r = null);
        return function e(r, n, o, i) {
            return t(r, n, function(t) {
                !t || "EMFILE" !== t.code && "ENFILE" !== t.code ? "function" == typeof o && o.apply(this, arguments) : fi([ e, [ r, n, o ], t, i || Date.now(), Date.now() ]);
            });
        }(e, r, n);
    };
    var r = e.writeFile;
    e.writeFile = function(e, t, n, o) {
        "function" == typeof n && (o = n, n = null);
        return function e(t, n, o, i, a) {
            return r(t, n, o, function(r) {
                !r || "EMFILE" !== r.code && "ENFILE" !== r.code ? "function" == typeof i && i.apply(this, arguments) : fi([ e, [ t, n, o, i ], r, a || Date.now(), Date.now() ]);
            });
        }(e, t, n, o);
    };
    var n = e.appendFile;
    n && (e.appendFile = function(e, t, r, o) {
        "function" == typeof r && (o = r, r = null);
        return function e(t, r, o, i, a) {
            return n(t, r, o, function(n) {
                !n || "EMFILE" !== n.code && "ENFILE" !== n.code ? "function" == typeof i && i.apply(this, arguments) : fi([ e, [ t, r, o, i ], n, a || Date.now(), Date.now() ]);
            });
        }(e, t, r, o);
    });
    var o = e.copyFile;
    o && (e.copyFile = function(e, t, r, n) {
        "function" == typeof r && (n = r, r = 0);
        return function e(t, r, n, i, a) {
            return o(t, r, n, function(o) {
                !o || "EMFILE" !== o.code && "ENFILE" !== o.code ? "function" == typeof i && i.apply(this, arguments) : fi([ e, [ t, r, n, i ], o, a || Date.now(), Date.now() ]);
            });
        }(e, t, r, n);
    });
    var i = e.readdir;
    e.readdir = function(e, t, r) {
        "function" == typeof t && (r = t, t = null);
        var n = a.test(process.version) ? function(e, t, r, n) {
            return i(e, o(e, t, r, n));
        } : function(e, t, r, n) {
            return i(e, t, o(e, t, r, n));
        };
        return n(e, t, r);
        function o(e, t, r, o) {
            return function(i, a) {
                !i || "EMFILE" !== i.code && "ENFILE" !== i.code ? (a && a.sort && a.sort(), "function" == typeof r && r.call(this, i, a)) : fi([ n, [ e, t, r ], i, o || Date.now(), Date.now() ]);
            };
        }
    };
    var a = /^v[0-5]\./;
    if ("v0.8" === process.version.substr(0, 4)) {
        var s = ri(e);
        d = s.ReadStream, p = s.WriteStream;
    }
    var u = e.ReadStream;
    u && (d.prototype = Object.create(u.prototype), d.prototype.open = function() {
        var e = this;
        v(e.path, e.flags, e.mode, function(t, r) {
            t ? (e.autoClose && e.destroy(), e.emit("error", t)) : (e.fd = r, e.emit("open", r), 
            e.read());
        });
    });
    var l = e.WriteStream;
    l && (p.prototype = Object.create(l.prototype), p.prototype.open = function() {
        var e = this;
        v(e.path, e.flags, e.mode, function(t, r) {
            t ? (e.destroy(), e.emit("error", t)) : (e.fd = r, e.emit("open", r));
        });
    }), Object.defineProperty(e, "ReadStream", {
        get: function() {
            return d;
        },
        set: function(e) {
            d = e;
        },
        enumerable: !0,
        configurable: !0
    }), Object.defineProperty(e, "WriteStream", {
        get: function() {
            return p;
        },
        set: function(e) {
            p = e;
        },
        enumerable: !0,
        configurable: !0
    });
    var c = d;
    Object.defineProperty(e, "FileReadStream", {
        get: function() {
            return c;
        },
        set: function(e) {
            c = e;
        },
        enumerable: !0,
        configurable: !0
    });
    var f = p;
    function d(e, t) {
        return this instanceof d ? (u.apply(this, arguments), this) : d.apply(Object.create(d.prototype), arguments);
    }
    function p(e, t) {
        return this instanceof p ? (l.apply(this, arguments), this) : p.apply(Object.create(p.prototype), arguments);
    }
    Object.defineProperty(e, "FileWriteStream", {
        get: function() {
            return f;
        },
        set: function(e) {
            f = e;
        },
        enumerable: !0,
        configurable: !0
    });
    var h = e.open;
    function v(e, t, r, n) {
        return "function" == typeof r && (n = r, r = null), function e(t, r, n, o, i) {
            return h(t, r, n, function(a, s) {
                !a || "EMFILE" !== a.code && "ENFILE" !== a.code ? "function" == typeof o && o.apply(this, arguments) : fi([ e, [ t, r, n, o ], a, i || Date.now(), Date.now() ]);
            });
        }(e, t, r, n);
    }
    return e.open = v, e;
}

function fi(e) {
    ai("ENQUEUE", e[0].name, e[1]), ei[Yo].push(e), pi();
}

function di() {
    for (var e = Date.now(), t = 0; t < ei[Yo].length; ++t) {
        ei[Yo][t].length > 2 && (ei[Yo][t][3] = e, ei[Yo][t][4] = e);
    }
    pi();
}

function pi() {
    if (clearTimeout(ui), ui = void 0, 0 !== ei[Yo].length) {
        var e = ei[Yo].shift(), t = e[0], r = e[1], n = e[2], o = e[3], i = e[4];
        if (void 0 === o) {
            ai("RETRY", t.name, r), t.apply(null, r);
        } else if (Date.now() - o >= 6e4) {
            ai("TIMEOUT", t.name, r);
            var a = r.pop();
            "function" == typeof a && a.call(null, n);
        } else {
            var s = Date.now() - i, u = Math.max(i - o, 1);
            s >= Math.min(1.2 * u, 100) ? (ai("RETRY", t.name, r), t.apply(null, r.concat([ o ]))) : ei[Yo].push(e);
        }
        void 0 === ui && (ui = setTimeout(pi, 0));
    }
}

process.env.TEST_GRACEFUL_FS_GLOBAL_PATCH && !ei.__patched && (li = ci(ei), ei.__patched = !0), 
function(e) {
    const t = zo.fromCallback, r = li, n = [ "access", "appendFile", "chmod", "chown", "close", "copyFile", "cp", "fchmod", "fchown", "fdatasync", "fstat", "fsync", "ftruncate", "futimes", "glob", "lchmod", "lchown", "lutimes", "link", "lstat", "mkdir", "mkdtemp", "open", "opendir", "readdir", "readFile", "readlink", "realpath", "rename", "rm", "rmdir", "stat", "statfs", "symlink", "truncate", "unlink", "utimes", "writeFile" ].filter(e => "function" == typeof r[e]);
    Object.assign(e, r), n.forEach(n => {
        e[n] = t(r[n]);
    }), e.exists = function(e, t) {
        return "function" == typeof t ? r.exists(e, t) : new Promise(t => r.exists(e, t));
    }, e.read = function(e, t, n, o, i, a) {
        return "function" == typeof a ? r.read(e, t, n, o, i, a) : new Promise((a, s) => {
            r.read(e, t, n, o, i, (e, t, r) => {
                if (e) {
                    return s(e);
                }
                a({
                    bytesRead: t,
                    buffer: r
                });
            });
        });
    }, e.write = function(e, t, ...n) {
        return "function" == typeof n[n.length - 1] ? r.write(e, t, ...n) : new Promise((o, i) => {
            r.write(e, t, ...n, (e, t, r) => {
                if (e) {
                    return i(e);
                }
                o({
                    bytesWritten: t,
                    buffer: r
                });
            });
        });
    }, e.readv = function(e, t, ...n) {
        return "function" == typeof n[n.length - 1] ? r.readv(e, t, ...n) : new Promise((o, i) => {
            r.readv(e, t, ...n, (e, t, r) => {
                if (e) {
                    return i(e);
                }
                o({
                    bytesRead: t,
                    buffers: r
                });
            });
        });
    }, e.writev = function(e, t, ...n) {
        return "function" == typeof n[n.length - 1] ? r.writev(e, t, ...n) : new Promise((o, i) => {
            r.writev(e, t, ...n, (e, t, r) => {
                if (e) {
                    return i(e);
                }
                o({
                    bytesWritten: t,
                    buffers: r
                });
            });
        });
    }, "function" == typeof r.realpath.native ? e.realpath.native = t(r.realpath.native) : process.emitWarning("fs.realpath.native is not a function. Is fs being monkey-patched?", "Warning", "fs-extra-WARN0003");
}(Uo);

var hi = {}, vi = {};

const gi = r;

vi.checkPath = function(e) {
    if ("win32" === process.platform) {
        if (/[<>:"|?*]/.test(e.replace(gi.parse(e).root, ""))) {
            const t = new Error(`Path contains invalid characters: ${e}`);
            throw t.code = "EINVAL", t;
        }
    }
};

const mi = Uo, {checkPath: yi} = vi, _i = e => "number" == typeof e ? e : {
    mode: 511,
    ...e
}.mode;

hi.makeDir = async (e, t) => (yi(e), mi.mkdir(e, {
    mode: _i(t),
    recursive: !0
})), hi.makeDirSync = (e, t) => (yi(e), mi.mkdirSync(e, {
    mode: _i(t),
    recursive: !0
}));

const Ei = zo.fromPromise, {makeDir: bi, makeDirSync: wi} = hi, Di = Ei(bi);

var Si = {
    mkdirs: Di,
    mkdirsSync: wi,
    mkdirp: Di,
    mkdirpSync: wi,
    ensureDir: Di,
    ensureDirSync: wi
};

const Ai = zo.fromPromise, Oi = Uo;

var Ci = {
    pathExists: Ai(function(e) {
        return Oi.access(e).then(() => !0).catch(() => !1);
    }),
    pathExistsSync: Oi.existsSync
};

const xi = Uo;

var Fi = {
    utimesMillis: (0, zo.fromPromise)(async function(e, t, r) {
        const n = await xi.open(e, "r+");
        let o = null;
        try {
            await xi.futimes(n, t, r);
        } finally {
            try {
                await xi.close(n);
            } catch (e) {
                o = e;
            }
        }
        if (o) {
            throw o;
        }
    }),
    utimesMillisSync: function(e, t, r) {
        const n = xi.openSync(e, "r+");
        return xi.futimesSync(n, t, r), xi.closeSync(n);
    }
};

const Mi = Uo, Pi = r, Ii = zo.fromPromise;

function ki(e, t) {
    return t.ino && t.dev && t.ino === e.ino && t.dev === e.dev;
}

function Ri(e, t) {
    const r = Pi.resolve(e).split(Pi.sep).filter(e => e), n = Pi.resolve(t).split(Pi.sep).filter(e => e);
    return r.every((e, t) => n[t] === e);
}

function Ti(e, t, r) {
    return `Cannot ${r} '${e}' to a subdirectory of itself, '${t}'.`;
}

var ji = {
    checkPaths: Ii(async function(e, t, r, n) {
        const {srcStat: o, destStat: i} = await function(e, t, r) {
            const n = r.dereference ? e => Mi.stat(e, {
                bigint: !0
            }) : e => Mi.lstat(e, {
                bigint: !0
            });
            return Promise.all([ n(e), n(t).catch(e => {
                if ("ENOENT" === e.code) {
                    return null;
                }
                throw e;
            }) ]).then(([e, t]) => ({
                srcStat: e,
                destStat: t
            }));
        }(e, t, n);
        if (i) {
            if (ki(o, i)) {
                const n = Pi.basename(e), a = Pi.basename(t);
                if ("move" === r && n !== a && n.toLowerCase() === a.toLowerCase()) {
                    return {
                        srcStat: o,
                        destStat: i,
                        isChangingCase: !0
                    };
                }
                throw new Error("Source and destination must not be the same.");
            }
            if (o.isDirectory() && !i.isDirectory()) {
                throw new Error(`Cannot overwrite non-directory '${t}' with directory '${e}'.`);
            }
            if (!o.isDirectory() && i.isDirectory()) {
                throw new Error(`Cannot overwrite directory '${t}' with non-directory '${e}'.`);
            }
        }
        if (o.isDirectory() && Ri(e, t)) {
            throw new Error(Ti(e, t, r));
        }
        return {
            srcStat: o,
            destStat: i
        };
    }),
    checkPathsSync: function(e, t, r, n) {
        const {srcStat: o, destStat: i} = function(e, t, r) {
            let n;
            const o = r.dereference ? e => Mi.statSync(e, {
                bigint: !0
            }) : e => Mi.lstatSync(e, {
                bigint: !0
            }), i = o(e);
            try {
                n = o(t);
            } catch (e) {
                if ("ENOENT" === e.code) {
                    return {
                        srcStat: i,
                        destStat: null
                    };
                }
                throw e;
            }
            return {
                srcStat: i,
                destStat: n
            };
        }(e, t, n);
        if (i) {
            if (ki(o, i)) {
                const n = Pi.basename(e), a = Pi.basename(t);
                if ("move" === r && n !== a && n.toLowerCase() === a.toLowerCase()) {
                    return {
                        srcStat: o,
                        destStat: i,
                        isChangingCase: !0
                    };
                }
                throw new Error("Source and destination must not be the same.");
            }
            if (o.isDirectory() && !i.isDirectory()) {
                throw new Error(`Cannot overwrite non-directory '${t}' with directory '${e}'.`);
            }
            if (!o.isDirectory() && i.isDirectory()) {
                throw new Error(`Cannot overwrite directory '${t}' with non-directory '${e}'.`);
            }
        }
        if (o.isDirectory() && Ri(e, t)) {
            throw new Error(Ti(e, t, r));
        }
        return {
            srcStat: o,
            destStat: i
        };
    },
    checkParentPaths: Ii(async function e(t, r, n, o) {
        const i = Pi.resolve(Pi.dirname(t)), a = Pi.resolve(Pi.dirname(n));
        if (a === i || a === Pi.parse(a).root) {
            return;
        }
        let s;
        try {
            s = await Mi.stat(a, {
                bigint: !0
            });
        } catch (e) {
            if ("ENOENT" === e.code) {
                return;
            }
            throw e;
        }
        if (ki(r, s)) {
            throw new Error(Ti(t, n, o));
        }
        return e(t, r, a, o);
    }),
    checkParentPathsSync: function e(t, r, n, o) {
        const i = Pi.resolve(Pi.dirname(t)), a = Pi.resolve(Pi.dirname(n));
        if (a === i || a === Pi.parse(a).root) {
            return;
        }
        let s;
        try {
            s = Mi.statSync(a, {
                bigint: !0
            });
        } catch (e) {
            if ("ENOENT" === e.code) {
                return;
            }
            throw e;
        }
        if (ki(r, s)) {
            throw new Error(Ti(t, n, o));
        }
        return e(t, r, a, o);
    },
    isSrcSubdir: Ri,
    areIdentical: ki
};

const Li = Uo, Ni = r, {mkdirs: Bi} = Si, {pathExists: Ui} = Ci, {utimesMillis: zi} = Fi, Hi = ji;

async function $i(e, t, r) {
    return !r.filter || r.filter(e, t);
}

async function Gi(e, t, r, n) {
    const o = n.dereference ? Li.stat : Li.lstat, i = await o(t);
    if (i.isDirectory()) {
        return async function(e, t, r, n, o) {
            t || await Li.mkdir(n);
            const i = [];
            for await (const e of await Li.opendir(r)) {
                const t = Ni.join(r, e.name), a = Ni.join(n, e.name);
                i.push($i(t, a, o).then(e => {
                    if (e) {
                        return Hi.checkPaths(t, a, "copy", o).then(({destStat: e}) => Gi(e, t, a, o));
                    }
                }));
            }
            await Promise.all(i), t || await Li.chmod(n, e.mode);
        }(i, e, t, r, n);
    }
    if (i.isFile() || i.isCharacterDevice() || i.isBlockDevice()) {
        return async function(e, t, r, n, o) {
            if (!t) {
                return Wi(e, r, n, o);
            }
            if (o.overwrite) {
                return await Li.unlink(n), Wi(e, r, n, o);
            }
            if (o.errorOnExist) {
                throw new Error(`'${n}' already exists`);
            }
        }(i, e, t, r, n);
    }
    if (i.isSymbolicLink()) {
        return async function(e, t, r, n) {
            let o = await Li.readlink(t);
            n.dereference && (o = Ni.resolve(process.cwd(), o));
            if (!e) {
                return Li.symlink(o, r);
            }
            let i = null;
            try {
                i = await Li.readlink(r);
            } catch (e) {
                if ("EINVAL" === e.code || "UNKNOWN" === e.code) {
                    return Li.symlink(o, r);
                }
                throw e;
            }
            n.dereference && (i = Ni.resolve(process.cwd(), i));
            if (Hi.isSrcSubdir(o, i)) {
                throw new Error(`Cannot copy '${o}' to a subdirectory of itself, '${i}'.`);
            }
            if (Hi.isSrcSubdir(i, o)) {
                throw new Error(`Cannot overwrite '${i}' with '${o}'.`);
            }
            return await Li.unlink(r), Li.symlink(o, r);
        }(e, t, r, n);
    }
    if (i.isSocket()) {
        throw new Error(`Cannot copy a socket file: ${t}`);
    }
    if (i.isFIFO()) {
        throw new Error(`Cannot copy a FIFO pipe: ${t}`);
    }
    throw new Error(`Unknown file: ${t}`);
}

async function Wi(e, t, r, n) {
    if (await Li.copyFile(t, r), n.preserveTimestamps) {
        128 & e.mode || await function(e, t) {
            return Li.chmod(e, 128 | t);
        }(r, e.mode);
        const n = await Li.stat(t);
        await zi(r, n.atime, n.mtime);
    }
    return Li.chmod(r, e.mode);
}

var Vi = async function(e, t, r = {}) {
    "function" == typeof r && (r = {
        filter: r
    }), r.clobber = !("clobber" in r) || !!r.clobber, r.overwrite = "overwrite" in r ? !!r.overwrite : r.clobber, 
    r.preserveTimestamps && "ia32" === process.arch && process.emitWarning("Using the preserveTimestamps option in 32-bit node is not recommended;\n\n\tsee https://github.com/jprichardson/node-fs-extra/issues/269", "Warning", "fs-extra-WARN0001");
    const {srcStat: n, destStat: o} = await Hi.checkPaths(e, t, "copy", r);
    if (await Hi.checkParentPaths(e, n, t, "copy"), !await $i(e, t, r)) {
        return;
    }
    const i = Ni.dirname(t);
    await Ui(i) || await Bi(i), await Gi(o, e, t, r);
};

const Ki = li, qi = r, Ji = Si.mkdirsSync, Xi = Fi.utimesMillisSync, Zi = ji;

function Yi(e, t, r, n) {
    const o = (n.dereference ? Ki.statSync : Ki.lstatSync)(t);
    if (o.isDirectory()) {
        return function(e, t, r, n, o) {
            return t ? ta(r, n, o) : function(e, t, r, n) {
                return Ki.mkdirSync(r), ta(t, r, n), ea(r, e);
            }(e.mode, r, n, o);
        }(o, e, t, r, n);
    }
    if (o.isFile() || o.isCharacterDevice() || o.isBlockDevice()) {
        return function(e, t, r, n, o) {
            return t ? function(e, t, r, n) {
                if (n.overwrite) {
                    return Ki.unlinkSync(r), Qi(e, t, r, n);
                }
                if (n.errorOnExist) {
                    throw new Error(`'${r}' already exists`);
                }
            }(e, r, n, o) : Qi(e, r, n, o);
        }(o, e, t, r, n);
    }
    if (o.isSymbolicLink()) {
        return function(e, t, r, n) {
            let o = Ki.readlinkSync(t);
            n.dereference && (o = qi.resolve(process.cwd(), o));
            if (e) {
                let e;
                try {
                    e = Ki.readlinkSync(r);
                } catch (e) {
                    if ("EINVAL" === e.code || "UNKNOWN" === e.code) {
                        return Ki.symlinkSync(o, r);
                    }
                    throw e;
                }
                if (n.dereference && (e = qi.resolve(process.cwd(), e)), Zi.isSrcSubdir(o, e)) {
                    throw new Error(`Cannot copy '${o}' to a subdirectory of itself, '${e}'.`);
                }
                if (Zi.isSrcSubdir(e, o)) {
                    throw new Error(`Cannot overwrite '${e}' with '${o}'.`);
                }
                return function(e, t) {
                    return Ki.unlinkSync(t), Ki.symlinkSync(e, t);
                }(o, r);
            }
            return Ki.symlinkSync(o, r);
        }(e, t, r, n);
    }
    if (o.isSocket()) {
        throw new Error(`Cannot copy a socket file: ${t}`);
    }
    if (o.isFIFO()) {
        throw new Error(`Cannot copy a FIFO pipe: ${t}`);
    }
    throw new Error(`Unknown file: ${t}`);
}

function Qi(e, t, r, n) {
    return Ki.copyFileSync(t, r), n.preserveTimestamps && function(e, t, r) {
        (function(e) {
            return !(128 & e);
        })(e) && function(e, t) {
            ea(e, 128 | t);
        }(r, e);
        (function(e, t) {
            const r = Ki.statSync(e);
            Xi(t, r.atime, r.mtime);
        })(t, r);
    }(e.mode, t, r), ea(r, e.mode);
}

function ea(e, t) {
    return Ki.chmodSync(e, t);
}

function ta(e, t, r) {
    const n = Ki.opendirSync(e);
    try {
        let o;
        for (;null !== (o = n.readSync()); ) {
            ra(o.name, e, t, r);
        }
    } finally {
        n.closeSync();
    }
}

function ra(e, t, r, n) {
    const o = qi.join(t, e), i = qi.join(r, e);
    if (n.filter && !n.filter(o, i)) {
        return;
    }
    const {destStat: a} = Zi.checkPathsSync(o, i, "copy", n);
    return Yi(a, o, i, n);
}

var na = function(e, t, r) {
    "function" == typeof r && (r = {
        filter: r
    }), (r = r || {}).clobber = !("clobber" in r) || !!r.clobber, r.overwrite = "overwrite" in r ? !!r.overwrite : r.clobber, 
    r.preserveTimestamps && "ia32" === process.arch && process.emitWarning("Using the preserveTimestamps option in 32-bit node is not recommended;\n\n\tsee https://github.com/jprichardson/node-fs-extra/issues/269", "Warning", "fs-extra-WARN0002");
    const {srcStat: n, destStat: o} = Zi.checkPathsSync(e, t, "copy", r);
    if (Zi.checkParentPathsSync(e, n, t, "copy"), r.filter && !r.filter(e, t)) {
        return;
    }
    const i = qi.dirname(t);
    return Ki.existsSync(i) || Ji(i), Yi(o, e, t, r);
};

var oa = {
    copy: (0, zo.fromPromise)(Vi),
    copySync: na
};

const ia = li;

var aa = {
    remove: (0, zo.fromCallback)(function(e, t) {
        ia.rm(e, {
            recursive: !0,
            force: !0
        }, t);
    }),
    removeSync: function(e) {
        ia.rmSync(e, {
            recursive: !0,
            force: !0
        });
    }
};

const sa = zo.fromPromise, ua = Uo, la = r, ca = Si, fa = aa, da = sa(async function(e) {
    let t;
    try {
        t = await ua.readdir(e);
    } catch {
        return ca.mkdirs(e);
    }
    return Promise.all(t.map(t => fa.remove(la.join(e, t))));
});

function pa(e) {
    let t;
    try {
        t = ua.readdirSync(e);
    } catch {
        return ca.mkdirsSync(e);
    }
    t.forEach(t => {
        t = la.join(e, t), fa.removeSync(t);
    });
}

var ha = {
    emptyDirSync: pa,
    emptydirSync: pa,
    emptyDir: da,
    emptydir: da
};

const va = zo.fromPromise, ga = r, ma = Uo, ya = Si;

var _a = {
    createFile: va(async function(e) {
        let t;
        try {
            t = await ma.stat(e);
        } catch {}
        if (t && t.isFile()) {
            return;
        }
        const r = ga.dirname(e);
        let n = null;
        try {
            n = await ma.stat(r);
        } catch (t) {
            if ("ENOENT" === t.code) {
                return await ya.mkdirs(r), void await ma.writeFile(e, "");
            }
            throw t;
        }
        n.isDirectory() ? await ma.writeFile(e, "") : await ma.readdir(r);
    }),
    createFileSync: function(e) {
        let t;
        try {
            t = ma.statSync(e);
        } catch {}
        if (t && t.isFile()) {
            return;
        }
        const r = ga.dirname(e);
        try {
            ma.statSync(r).isDirectory() || ma.readdirSync(r);
        } catch (e) {
            if (!e || "ENOENT" !== e.code) {
                throw e;
            }
            ya.mkdirsSync(r);
        }
        ma.writeFileSync(e, "");
    }
};

const Ea = zo.fromPromise, ba = r, wa = Uo, Da = Si, {pathExists: Sa} = Ci, {areIdentical: Aa} = ji;

var Oa = {
    createLink: Ea(async function(e, t) {
        let r, n;
        try {
            r = await wa.lstat(t);
        } catch {}
        try {
            n = await wa.lstat(e);
        } catch (e) {
            throw e.message = e.message.replace("lstat", "ensureLink"), e;
        }
        if (r && Aa(n, r)) {
            return;
        }
        const o = ba.dirname(t);
        await Sa(o) || await Da.mkdirs(o), await wa.link(e, t);
    }),
    createLinkSync: function(e, t) {
        let r;
        try {
            r = wa.lstatSync(t);
        } catch {}
        try {
            const t = wa.lstatSync(e);
            if (r && Aa(t, r)) {
                return;
            }
        } catch (e) {
            throw e.message = e.message.replace("lstat", "ensureLink"), e;
        }
        const n = ba.dirname(t);
        return wa.existsSync(n) || Da.mkdirsSync(n), wa.linkSync(e, t);
    }
};

const Ca = r, xa = Uo, {pathExists: Fa} = Ci;

var Ma = {
    symlinkPaths: (0, zo.fromPromise)(async function(e, t) {
        if (Ca.isAbsolute(e)) {
            try {
                await xa.lstat(e);
            } catch (e) {
                throw e.message = e.message.replace("lstat", "ensureSymlink"), e;
            }
            return {
                toCwd: e,
                toDst: e
            };
        }
        const r = Ca.dirname(t), n = Ca.join(r, e);
        if (await Fa(n)) {
            return {
                toCwd: n,
                toDst: e
            };
        }
        try {
            await xa.lstat(e);
        } catch (e) {
            throw e.message = e.message.replace("lstat", "ensureSymlink"), e;
        }
        return {
            toCwd: e,
            toDst: Ca.relative(r, e)
        };
    }),
    symlinkPathsSync: function(e, t) {
        if (Ca.isAbsolute(e)) {
            if (!xa.existsSync(e)) {
                throw new Error("absolute srcpath does not exist");
            }
            return {
                toCwd: e,
                toDst: e
            };
        }
        const r = Ca.dirname(t), n = Ca.join(r, e);
        if (xa.existsSync(n)) {
            return {
                toCwd: n,
                toDst: e
            };
        }
        if (!xa.existsSync(e)) {
            throw new Error("relative srcpath does not exist");
        }
        return {
            toCwd: e,
            toDst: Ca.relative(r, e)
        };
    }
};

const Pa = Uo;

var Ia = {
    symlinkType: (0, zo.fromPromise)(async function(e, t) {
        if (t) {
            return t;
        }
        let r;
        try {
            r = await Pa.lstat(e);
        } catch {
            return "file";
        }
        return r && r.isDirectory() ? "dir" : "file";
    }),
    symlinkTypeSync: function(e, t) {
        if (t) {
            return t;
        }
        let r;
        try {
            r = Pa.lstatSync(e);
        } catch {
            return "file";
        }
        return r && r.isDirectory() ? "dir" : "file";
    }
};

const ka = zo.fromPromise, Ra = r, Ta = Uo, {mkdirs: ja, mkdirsSync: La} = Si, {symlinkPaths: Na, symlinkPathsSync: Ba} = Ma, {symlinkType: Ua, symlinkTypeSync: za} = Ia, {pathExists: Ha} = Ci, {areIdentical: $a} = ji;

var Ga = {
    createSymlink: ka(async function(e, t, r) {
        let n;
        try {
            n = await Ta.lstat(t);
        } catch {}
        if (n && n.isSymbolicLink()) {
            const [r, n] = await Promise.all([ Ta.stat(e), Ta.stat(t) ]);
            if ($a(r, n)) {
                return;
            }
        }
        const o = await Na(e, t);
        e = o.toDst;
        const i = await Ua(o.toCwd, r), a = Ra.dirname(t);
        return await Ha(a) || await ja(a), Ta.symlink(e, t, i);
    }),
    createSymlinkSync: function(e, t, r) {
        let n;
        try {
            n = Ta.lstatSync(t);
        } catch {}
        if (n && n.isSymbolicLink()) {
            const r = Ta.statSync(e), n = Ta.statSync(t);
            if ($a(r, n)) {
                return;
            }
        }
        const o = Ba(e, t);
        e = o.toDst, r = za(o.toCwd, r);
        const i = Ra.dirname(t);
        return Ta.existsSync(i) || La(i), Ta.symlinkSync(e, t, r);
    }
};

const {createFile: Wa, createFileSync: Va} = _a, {createLink: Ka, createLinkSync: qa} = Oa, {createSymlink: Ja, createSymlinkSync: Xa} = Ga;

var Za = {
    createFile: Wa,
    createFileSync: Va,
    ensureFile: Wa,
    ensureFileSync: Va,
    createLink: Ka,
    createLinkSync: qa,
    ensureLink: Ka,
    ensureLinkSync: qa,
    createSymlink: Ja,
    createSymlinkSync: Xa,
    ensureSymlink: Ja,
    ensureSymlinkSync: Xa
};

var Ya = {
    stringify: function(e, {EOL: t = "\n", finalEOL: r = !0, replacer: n = null, spaces: o} = {}) {
        const i = r ? t : "";
        return JSON.stringify(e, n, o).replace(/\n/g, t) + i;
    },
    stripBom: function(e) {
        return Buffer.isBuffer(e) && (e = e.toString("utf8")), e.replace(/^\uFEFF/, "");
    }
};

let Qa;

try {
    Qa = li;
} catch (Bk) {
    Qa = t;
}

const es = zo, {stringify: ts, stripBom: rs} = Ya;

const ns = es.fromPromise(async function(e, t = {}) {
    "string" == typeof t && (t = {
        encoding: t
    });
    const r = t.fs || Qa, n = !("throws" in t) || t.throws;
    let o, i = await es.fromCallback(r.readFile)(e, t);
    i = rs(i);
    try {
        o = JSON.parse(i, t ? t.reviver : null);
    } catch (t) {
        if (n) {
            throw t.message = `${e}: ${t.message}`, t;
        }
        return null;
    }
    return o;
});

const os = es.fromPromise(async function(e, t, r = {}) {
    const n = r.fs || Qa, o = ts(t, r);
    await es.fromCallback(n.writeFile)(e, o, r);
});

const is = {
    readFile: ns,
    readFileSync: function(e, t = {}) {
        "string" == typeof t && (t = {
            encoding: t
        });
        const r = t.fs || Qa, n = !("throws" in t) || t.throws;
        try {
            let n = r.readFileSync(e, t);
            return n = rs(n), JSON.parse(n, t.reviver);
        } catch (t) {
            if (n) {
                throw t.message = `${e}: ${t.message}`, t;
            }
            return null;
        }
    },
    writeFile: os,
    writeFileSync: function(e, t, r = {}) {
        const n = r.fs || Qa, o = ts(t, r);
        return n.writeFileSync(e, o, r);
    }
};

var as = {
    readJson: is.readFile,
    readJsonSync: is.readFileSync,
    writeJson: is.writeFile,
    writeJsonSync: is.writeFileSync
};

const ss = zo.fromPromise, us = Uo, ls = r, cs = Si, fs = Ci.pathExists;

var ds = {
    outputFile: ss(async function(e, t, r = "utf-8") {
        const n = ls.dirname(e);
        return await fs(n) || await cs.mkdirs(n), us.writeFile(e, t, r);
    }),
    outputFileSync: function(e, ...t) {
        const r = ls.dirname(e);
        us.existsSync(r) || cs.mkdirsSync(r), us.writeFileSync(e, ...t);
    }
};

const {stringify: ps} = Ya, {outputFile: hs} = ds;

var vs = async function(e, t, r = {}) {
    const n = ps(t, r);
    await hs(e, n, r);
};

const {stringify: gs} = Ya, {outputFileSync: ms} = ds;

var ys = function(e, t, r) {
    const n = gs(t, r);
    ms(e, n, r);
};

const _s = zo.fromPromise, Es = as;

Es.outputJson = _s(vs), Es.outputJsonSync = ys, Es.outputJSON = Es.outputJson, Es.outputJSONSync = Es.outputJsonSync, 
Es.writeJSON = Es.writeJson, Es.writeJSONSync = Es.writeJsonSync, Es.readJSON = Es.readJson, 
Es.readJSONSync = Es.readJsonSync;

var bs = Es;

const ws = Uo, Ds = r, {copy: Ss} = oa, {remove: As} = aa, {mkdirp: Os} = Si, {pathExists: Cs} = Ci, xs = ji;

var Fs = async function(e, t, r = {}) {
    const n = r.overwrite || r.clobber || !1, {srcStat: o, isChangingCase: i = !1} = await xs.checkPaths(e, t, "move", r);
    await xs.checkParentPaths(e, o, t, "move");
    const a = Ds.dirname(t);
    return Ds.parse(a).root !== a && await Os(a), async function(e, t, r, n) {
        if (!n) {
            if (r) {
                await As(t);
            } else if (await Cs(t)) {
                throw new Error("dest already exists.");
            }
        }
        try {
            await ws.rename(e, t);
        } catch (n) {
            if ("EXDEV" !== n.code) {
                throw n;
            }
            await async function(e, t, r) {
                const n = {
                    overwrite: r,
                    errorOnExist: !0,
                    preserveTimestamps: !0
                };
                return await Ss(e, t, n), As(e);
            }(e, t, r);
        }
    }(e, t, n, i);
};

const Ms = li, Ps = r, Is = oa.copySync, ks = aa.removeSync, Rs = Si.mkdirpSync, Ts = ji;

function js(e, t, r) {
    try {
        Ms.renameSync(e, t);
    } catch (n) {
        if ("EXDEV" !== n.code) {
            throw n;
        }
        return function(e, t, r) {
            const n = {
                overwrite: r,
                errorOnExist: !0,
                preserveTimestamps: !0
            };
            return Is(e, t, n), ks(e);
        }(e, t, r);
    }
}

var Ls = function(e, t, r) {
    const n = (r = r || {}).overwrite || r.clobber || !1, {srcStat: o, isChangingCase: i = !1} = Ts.checkPathsSync(e, t, "move", r);
    return Ts.checkParentPathsSync(e, o, t, "move"), function(e) {
        const t = Ps.dirname(e);
        return Ps.parse(t).root === t;
    }(t) || Rs(Ps.dirname(t)), function(e, t, r, n) {
        if (n) {
            return js(e, t, r);
        }
        if (r) {
            return ks(t), js(e, t, r);
        }
        if (Ms.existsSync(t)) {
            throw new Error("dest already exists.");
        }
        return js(e, t, r);
    }(e, t, n, i);
};

var Ns = {
    move: (0, zo.fromPromise)(Fs),
    moveSync: Ls
}, Bs = {
    ...Uo,
    ...oa,
    ...ha,
    ...Za,
    ...bs,
    ...Si,
    ...Ns,
    ...ds,
    ...Ci,
    ...aa
};

!function(e) {
    var n = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.readFileInZIP = e.addFilesToZip = e.getUint8ArrayReader = e.getZipWriter = e.getZipReader = void 0;
    const o = n(t), i = n(O), a = n(Bs), s = n(r);
    function u(e) {
        return new i.default.ZipReader(new i.default.BlobReader(e));
    }
    function l(e) {
        return new i.default.Uint8ArrayReader(new Uint8Array(a.default.readFileSync(e)));
    }
    e.getZipReader = u, e.getZipWriter = function(e) {
        return new i.default.ZipWriter(new i.default.BlobWriter(e));
    }, e.getUint8ArrayReader = l, e.addFilesToZip = async function e(t, r, n, o = "") {
        const i = a.default.readdirSync(r, {
            withFileTypes: !0
        });
        for (const u of i) {
            const i = s.default.join(r, u.name), c = s.default.posix.join(o, u.name);
            if (u.isSymbolicLink()) {
                const r = a.default.realpathSync(i);
                if (n.has(r)) {
                    continue;
                }
                n.add(r), await e(t, r, n, c);
            } else {
                u.isDirectory() ? await e(t, i, n, c) : u.isFile() && await t.add(c, l(i));
            }
        }
    }, e.readFileInZIP = async function(e, t) {
        const r = new Map, n = o.default.readFileSync(e), a = u(new Blob([ n ]));
        try {
            const e = await a.getEntries();
            for (const n of e) {
                if (!n.directory && t.includes(n.filename)) {
                    const e = n.getData && await n.getData(new i.default.TextWriter);
                    r.set(n.filename, e || "");
                }
            }
        } finally {
            a && await a.close();
        }
        return r;
    };
}(A), function(e) {
    var t = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.addFilesToZip = e.readFileInZIP = e.getZipWriter = e.getZipReader = e.maxPathLength = e.isLinux = e.isMac = e.isWindows = e.getOsLanguage = e.countryEnum = e.config = e.hashFile = e.hash = e.createHash = e.isSubPath = e.isCI = void 0;
    var r = _;
    Object.defineProperty(e, "isCI", {
        enumerable: !0,
        get: function() {
            return r.isCI;
        }
    });
    var n = E;
    Object.defineProperty(e, "isSubPath", {
        enumerable: !0,
        get: function() {
            return n.isSubPath;
        }
    });
    var o = b;
    Object.defineProperty(e, "createHash", {
        enumerable: !0,
        get: function() {
            return o.createHash;
        }
    }), Object.defineProperty(e, "hash", {
        enumerable: !0,
        get: function() {
            return o.hash;
        }
    }), Object.defineProperty(e, "hashFile", {
        enumerable: !0,
        get: function() {
            return o.hashFile;
        }
    });
    var i = w;
    Object.defineProperty(e, "config", {
        enumerable: !0,
        get: function() {
            return t(i).default;
        }
    });
    var a = D;
    Object.defineProperty(e, "countryEnum", {
        enumerable: !0,
        get: function() {
            return a.countryEnum;
        }
    }), Object.defineProperty(e, "getOsLanguage", {
        enumerable: !0,
        get: function() {
            return a.getOsLanguage;
        }
    });
    var s = S;
    Object.defineProperty(e, "isWindows", {
        enumerable: !0,
        get: function() {
            return s.isWindows;
        }
    }), Object.defineProperty(e, "isMac", {
        enumerable: !0,
        get: function() {
            return s.isMac;
        }
    }), Object.defineProperty(e, "isLinux", {
        enumerable: !0,
        get: function() {
            return s.isLinux;
        }
    }), Object.defineProperty(e, "maxPathLength", {
        enumerable: !0,
        get: function() {
            return s.maxPathLength;
        }
    });
    var u = A;
    Object.defineProperty(e, "getZipReader", {
        enumerable: !0,
        get: function() {
            return u.getZipReader;
        }
    }), Object.defineProperty(e, "getZipWriter", {
        enumerable: !0,
        get: function() {
            return u.getZipWriter;
        }
    }), Object.defineProperty(e, "readFileInZIP", {
        enumerable: !0,
        get: function() {
            return u.readFileInZIP;
        }
    }), Object.defineProperty(e, "addFilesToZip", {
        enumerable: !0,
        get: function() {
            return u.addFilesToZip;
        }
    });
}(y);

var Us = {
    exports: {}
};

var zs = {
    MAX_LENGTH: 256,
    MAX_SAFE_COMPONENT_LENGTH: 16,
    MAX_SAFE_BUILD_LENGTH: 250,
    MAX_SAFE_INTEGER: Number.MAX_SAFE_INTEGER || 9007199254740991,
    RELEASE_TYPES: [ "major", "premajor", "minor", "preminor", "patch", "prepatch", "prerelease" ],
    SEMVER_SPEC_VERSION: "2.0.0",
    FLAG_INCLUDE_PRERELEASE: 1,
    FLAG_LOOSE: 2
};

var Hs = "object" == typeof process && process.env && process.env.NODE_DEBUG && /\bsemver\b/i.test(process.env.NODE_DEBUG) ? (...e) => console.error("SEMVER", ...e) : () => {};

!function(e, t) {
    const {MAX_SAFE_COMPONENT_LENGTH: r, MAX_SAFE_BUILD_LENGTH: n, MAX_LENGTH: o} = zs, i = Hs, a = (t = e.exports = {}).re = [], s = t.safeRe = [], u = t.src = [], l = t.t = {};
    let c = 0;
    const f = "[a-zA-Z0-9-]", d = [ [ "\\s", 1 ], [ "\\d", o ], [ f, n ] ], p = (e, t, r) => {
        const n = (e => {
            for (const [t, r] of d) {
                e = e.split(`${t}*`).join(`${t}{0,${r}}`).split(`${t}+`).join(`${t}{1,${r}}`);
            }
            return e;
        })(t), o = c++;
        i(e, o, t), l[e] = o, u[o] = t, a[o] = new RegExp(t, r ? "g" : void 0), s[o] = new RegExp(n, r ? "g" : void 0);
    };
    p("NUMERICIDENTIFIER", "0|[1-9]\\d*"), p("NUMERICIDENTIFIERLOOSE", "\\d+"), p("NONNUMERICIDENTIFIER", `\\d*[a-zA-Z-]${f}*`), 
    p("MAINVERSION", `(${u[l.NUMERICIDENTIFIER]})\\.(${u[l.NUMERICIDENTIFIER]})\\.(${u[l.NUMERICIDENTIFIER]})`), 
    p("MAINVERSIONLOOSE", `(${u[l.NUMERICIDENTIFIERLOOSE]})\\.(${u[l.NUMERICIDENTIFIERLOOSE]})\\.(${u[l.NUMERICIDENTIFIERLOOSE]})`), 
    p("PRERELEASEIDENTIFIER", `(?:${u[l.NUMERICIDENTIFIER]}|${u[l.NONNUMERICIDENTIFIER]})`), 
    p("PRERELEASEIDENTIFIERLOOSE", `(?:${u[l.NUMERICIDENTIFIERLOOSE]}|${u[l.NONNUMERICIDENTIFIER]})`), 
    p("PRERELEASE", `(?:-(${u[l.PRERELEASEIDENTIFIER]}(?:\\.${u[l.PRERELEASEIDENTIFIER]})*))`), 
    p("PRERELEASELOOSE", `(?:-?(${u[l.PRERELEASEIDENTIFIERLOOSE]}(?:\\.${u[l.PRERELEASEIDENTIFIERLOOSE]})*))`), 
    p("BUILDIDENTIFIER", `${f}+`), p("BUILD", `(?:\\+(${u[l.BUILDIDENTIFIER]}(?:\\.${u[l.BUILDIDENTIFIER]})*))`), 
    p("FULLPLAIN", `v?${u[l.MAINVERSION]}${u[l.PRERELEASE]}?${u[l.BUILD]}?`), p("FULL", `^${u[l.FULLPLAIN]}$`), 
    p("LOOSEPLAIN", `[v=\\s]*${u[l.MAINVERSIONLOOSE]}${u[l.PRERELEASELOOSE]}?${u[l.BUILD]}?`), 
    p("LOOSE", `^${u[l.LOOSEPLAIN]}$`), p("GTLT", "((?:<|>)?=?)"), p("XRANGEIDENTIFIERLOOSE", `${u[l.NUMERICIDENTIFIERLOOSE]}|x|X|\\*`), 
    p("XRANGEIDENTIFIER", `${u[l.NUMERICIDENTIFIER]}|x|X|\\*`), p("XRANGEPLAIN", `[v=\\s]*(${u[l.XRANGEIDENTIFIER]})(?:\\.(${u[l.XRANGEIDENTIFIER]})(?:\\.(${u[l.XRANGEIDENTIFIER]})(?:${u[l.PRERELEASE]})?${u[l.BUILD]}?)?)?`), 
    p("XRANGEPLAINLOOSE", `[v=\\s]*(${u[l.XRANGEIDENTIFIERLOOSE]})(?:\\.(${u[l.XRANGEIDENTIFIERLOOSE]})(?:\\.(${u[l.XRANGEIDENTIFIERLOOSE]})(?:${u[l.PRERELEASELOOSE]})?${u[l.BUILD]}?)?)?`), 
    p("XRANGE", `^${u[l.GTLT]}\\s*${u[l.XRANGEPLAIN]}$`), p("XRANGELOOSE", `^${u[l.GTLT]}\\s*${u[l.XRANGEPLAINLOOSE]}$`), 
    p("COERCE", `(^|[^\\d])(\\d{1,${r}})(?:\\.(\\d{1,${r}}))?(?:\\.(\\d{1,${r}}))?(?:$|[^\\d])`), 
    p("COERCERTL", u[l.COERCE], !0), p("LONETILDE", "(?:~>?)"), p("TILDETRIM", `(\\s*)${u[l.LONETILDE]}\\s+`, !0), 
    t.tildeTrimReplace = "$1~", p("TILDE", `^${u[l.LONETILDE]}${u[l.XRANGEPLAIN]}$`), 
    p("TILDELOOSE", `^${u[l.LONETILDE]}${u[l.XRANGEPLAINLOOSE]}$`), p("LONECARET", "(?:\\^)"), 
    p("CARETTRIM", `(\\s*)${u[l.LONECARET]}\\s+`, !0), t.caretTrimReplace = "$1^", p("CARET", `^${u[l.LONECARET]}${u[l.XRANGEPLAIN]}$`), 
    p("CARETLOOSE", `^${u[l.LONECARET]}${u[l.XRANGEPLAINLOOSE]}$`), p("COMPARATORLOOSE", `^${u[l.GTLT]}\\s*(${u[l.LOOSEPLAIN]})$|^$`), 
    p("COMPARATOR", `^${u[l.GTLT]}\\s*(${u[l.FULLPLAIN]})$|^$`), p("COMPARATORTRIM", `(\\s*)${u[l.GTLT]}\\s*(${u[l.LOOSEPLAIN]}|${u[l.XRANGEPLAIN]})`, !0), 
    t.comparatorTrimReplace = "$1$2$3", p("HYPHENRANGE", `^\\s*(${u[l.XRANGEPLAIN]})\\s+-\\s+(${u[l.XRANGEPLAIN]})\\s*$`), 
    p("HYPHENRANGELOOSE", `^\\s*(${u[l.XRANGEPLAINLOOSE]})\\s+-\\s+(${u[l.XRANGEPLAINLOOSE]})\\s*$`), 
    p("STAR", "(<|>)?=?\\s*\\*"), p("GTE0", "^\\s*>=\\s*0\\.0\\.0\\s*$"), p("GTE0PRE", "^\\s*>=\\s*0\\.0\\.0-0\\s*$");
}(Us, Us.exports);

var $s = Us.exports;

const Gs = Object.freeze({
    loose: !0
}), Ws = Object.freeze({});

var Vs = e => e ? "object" != typeof e ? Gs : e : Ws;

const Ks = /^[0-9]+$/, qs = (e, t) => {
    const r = Ks.test(e), n = Ks.test(t);
    return r && n && (e = +e, t = +t), e === t ? 0 : r && !n ? -1 : n && !r ? 1 : e < t ? -1 : 1;
};

var Js = {
    compareIdentifiers: qs,
    rcompareIdentifiers: (e, t) => qs(t, e)
};

const Xs = Hs, {MAX_LENGTH: Zs, MAX_SAFE_INTEGER: Ys} = zs, {safeRe: Qs, t: eu} = $s, tu = Vs, {compareIdentifiers: ru} = Js;

var nu = class e {
    constructor(t, r) {
        if (r = tu(r), t instanceof e) {
            if (t.loose === !!r.loose && t.includePrerelease === !!r.includePrerelease) {
                return t;
            }
            t = t.version;
        } else if ("string" != typeof t) {
            throw new TypeError(`Invalid version. Must be a string. Got type "${typeof t}".`);
        }
        if (t.length > Zs) {
            throw new TypeError(`version is longer than ${Zs} characters`);
        }
        Xs("SemVer", t, r), this.options = r, this.loose = !!r.loose, this.includePrerelease = !!r.includePrerelease;
        const n = t.trim().match(r.loose ? Qs[eu.LOOSE] : Qs[eu.FULL]);
        if (!n) {
            throw new TypeError(`Invalid Version: ${t}`);
        }
        if (this.raw = t, this.major = +n[1], this.minor = +n[2], this.patch = +n[3], this.major > Ys || this.major < 0) {
            throw new TypeError("Invalid major version");
        }
        if (this.minor > Ys || this.minor < 0) {
            throw new TypeError("Invalid minor version");
        }
        if (this.patch > Ys || this.patch < 0) {
            throw new TypeError("Invalid patch version");
        }
        n[4] ? this.prerelease = n[4].split(".").map(e => {
            if (/^[0-9]+$/.test(e)) {
                const t = +e;
                if (t >= 0 && t < Ys) {
                    return t;
                }
            }
            return e;
        }) : this.prerelease = [], this.build = n[5] ? n[5].split(".") : [], this.format();
    }
    format() {
        return this.version = `${this.major}.${this.minor}.${this.patch}`, this.prerelease.length && (this.version += `-${this.prerelease.join(".")}`), 
        this.version;
    }
    toString() {
        return this.version;
    }
    compare(t) {
        if (Xs("SemVer.compare", this.version, this.options, t), !(t instanceof e)) {
            if ("string" == typeof t && t === this.version) {
                return 0;
            }
            t = new e(t, this.options);
        }
        return t.version === this.version ? 0 : this.compareMain(t) || this.comparePre(t);
    }
    compareMain(t) {
        return t instanceof e || (t = new e(t, this.options)), ru(this.major, t.major) || ru(this.minor, t.minor) || ru(this.patch, t.patch);
    }
    comparePre(t) {
        if (t instanceof e || (t = new e(t, this.options)), this.prerelease.length && !t.prerelease.length) {
            return -1;
        }
        if (!this.prerelease.length && t.prerelease.length) {
            return 1;
        }
        if (!this.prerelease.length && !t.prerelease.length) {
            return 0;
        }
        let r = 0;
        do {
            const e = this.prerelease[r], n = t.prerelease[r];
            if (Xs("prerelease compare", r, e, n), void 0 === e && void 0 === n) {
                return 0;
            }
            if (void 0 === n) {
                return 1;
            }
            if (void 0 === e) {
                return -1;
            }
            if (e !== n) {
                return ru(e, n);
            }
        } while (++r);
    }
    compareBuild(t) {
        t instanceof e || (t = new e(t, this.options));
        let r = 0;
        do {
            const e = this.build[r], n = t.build[r];
            if (Xs("prerelease compare", r, e, n), void 0 === e && void 0 === n) {
                return 0;
            }
            if (void 0 === n) {
                return 1;
            }
            if (void 0 === e) {
                return -1;
            }
            if (e !== n) {
                return ru(e, n);
            }
        } while (++r);
    }
    inc(e, t, r) {
        switch (e) {
          case "premajor":
            this.prerelease.length = 0, this.patch = 0, this.minor = 0, this.major++, this.inc("pre", t, r);
            break;

          case "preminor":
            this.prerelease.length = 0, this.patch = 0, this.minor++, this.inc("pre", t, r);
            break;

          case "prepatch":
            this.prerelease.length = 0, this.inc("patch", t, r), this.inc("pre", t, r);
            break;

          case "prerelease":
            0 === this.prerelease.length && this.inc("patch", t, r), this.inc("pre", t, r);
            break;

          case "major":
            0 === this.minor && 0 === this.patch && 0 !== this.prerelease.length || this.major++, 
            this.minor = 0, this.patch = 0, this.prerelease = [];
            break;

          case "minor":
            0 === this.patch && 0 !== this.prerelease.length || this.minor++, this.patch = 0, 
            this.prerelease = [];
            break;

          case "patch":
            0 === this.prerelease.length && this.patch++, this.prerelease = [];
            break;

          case "pre":
            {
                const e = Number(r) ? 1 : 0;
                if (!t && !1 === r) {
                    throw new Error("invalid increment argument: identifier is empty");
                }
                if (0 === this.prerelease.length) {
                    this.prerelease = [ e ];
                } else {
                    let n = this.prerelease.length;
                    for (;--n >= 0; ) {
                        "number" == typeof this.prerelease[n] && (this.prerelease[n]++, n = -2);
                    }
                    if (-1 === n) {
                        if (t === this.prerelease.join(".") && !1 === r) {
                            throw new Error("invalid increment argument: identifier already exists");
                        }
                        this.prerelease.push(e);
                    }
                }
                if (t) {
                    let n = [ t, e ];
                    !1 === r && (n = [ t ]), 0 === ru(this.prerelease[0], t) ? isNaN(this.prerelease[1]) && (this.prerelease = n) : this.prerelease = n;
                }
                break;
            }

          default:
            throw new Error(`invalid increment argument: ${e}`);
        }
        return this.raw = this.format(), this.build.length && (this.raw += `+${this.build.join(".")}`), 
        this;
    }
};

const ou = nu;

var iu = (e, t, r = !1) => {
    if (e instanceof ou) {
        return e;
    }
    try {
        return new ou(e, t);
    } catch (e) {
        if (!r) {
            return null;
        }
        throw e;
    }
};

const au = iu;

var su = (e, t) => {
    const r = au(e, t);
    return r ? r.version : null;
};

const uu = iu;

var lu = (e, t) => {
    const r = uu(e.trim().replace(/^[=v]+/, ""), t);
    return r ? r.version : null;
};

const cu = nu;

var fu = (e, t, r, n, o) => {
    "string" == typeof r && (o = n, n = r, r = void 0);
    try {
        return new cu(e instanceof cu ? e.version : e, r).inc(t, n, o).version;
    } catch (e) {
        return null;
    }
};

const du = iu;

var pu = (e, t) => {
    const r = du(e, null, !0), n = du(t, null, !0), o = r.compare(n);
    if (0 === o) {
        return null;
    }
    const i = o > 0, a = i ? r : n, s = i ? n : r, u = !!a.prerelease.length;
    if (!!s.prerelease.length && !u) {
        return s.patch || s.minor ? a.patch ? "patch" : a.minor ? "minor" : "major" : "major";
    }
    const l = u ? "pre" : "";
    return r.major !== n.major ? l + "major" : r.minor !== n.minor ? l + "minor" : r.patch !== n.patch ? l + "patch" : "prerelease";
};

const hu = nu;

var vu = (e, t) => new hu(e, t).major;

const gu = nu;

var mu = (e, t) => new gu(e, t).minor;

const yu = nu;

var _u = (e, t) => new yu(e, t).patch;

const Eu = iu;

var bu = (e, t) => {
    const r = Eu(e, t);
    return r && r.prerelease.length ? r.prerelease : null;
};

const wu = nu;

var Du = (e, t, r) => new wu(e, r).compare(new wu(t, r));

const Su = Du;

var Au = (e, t, r) => Su(t, e, r);

const Ou = Du;

var Cu = (e, t) => Ou(e, t, !0);

const xu = nu;

var Fu = (e, t, r) => {
    const n = new xu(e, r), o = new xu(t, r);
    return n.compare(o) || n.compareBuild(o);
};

const Mu = Fu;

var Pu = (e, t) => e.sort((e, r) => Mu(e, r, t));

const Iu = Fu;

var ku = (e, t) => e.sort((e, r) => Iu(r, e, t));

const Ru = Du;

var Tu = (e, t, r) => Ru(e, t, r) > 0;

const ju = Du;

var Lu = (e, t, r) => ju(e, t, r) < 0;

const Nu = Du;

var Bu = (e, t, r) => 0 === Nu(e, t, r);

const Uu = Du;

var zu = (e, t, r) => 0 !== Uu(e, t, r);

const Hu = Du;

var $u = (e, t, r) => Hu(e, t, r) >= 0;

const Gu = Du;

var Wu = (e, t, r) => Gu(e, t, r) <= 0;

const Vu = Bu, Ku = zu, qu = Tu, Ju = $u, Xu = Lu, Zu = Wu;

var Yu = (e, t, r, n) => {
    switch (t) {
      case "===":
        return "object" == typeof e && (e = e.version), "object" == typeof r && (r = r.version), 
        e === r;

      case "!==":
        return "object" == typeof e && (e = e.version), "object" == typeof r && (r = r.version), 
        e !== r;

      case "":
      case "=":
      case "==":
        return Vu(e, r, n);

      case "!=":
        return Ku(e, r, n);

      case ">":
        return qu(e, r, n);

      case ">=":
        return Ju(e, r, n);

      case "<":
        return Xu(e, r, n);

      case "<=":
        return Zu(e, r, n);

      default:
        throw new TypeError(`Invalid operator: ${t}`);
    }
};

const Qu = nu, el = iu, {safeRe: tl, t: rl} = $s;

var nl, ol, il, al, sl, ul, ll, cl, fl, dl, pl = (e, t) => {
    if (e instanceof Qu) {
        return e;
    }
    if ("number" == typeof e && (e = String(e)), "string" != typeof e) {
        return null;
    }
    let r = null;
    if ((t = t || {}).rtl) {
        let t;
        for (;(t = tl[rl.COERCERTL].exec(e)) && (!r || r.index + r[0].length !== e.length); ) {
            r && t.index + t[0].length === r.index + r[0].length || (r = t), tl[rl.COERCERTL].lastIndex = t.index + t[1].length + t[2].length;
        }
        tl[rl.COERCERTL].lastIndex = -1;
    } else {
        r = e.match(tl[rl.COERCE]);
    }
    return null === r ? null : el(`${r[2]}.${r[3] || "0"}.${r[4] || "0"}`, t);
};

function hl() {
    if (al) {
        return il;
    }
    function e(t) {
        var r = this;
        if (r instanceof e || (r = new e), r.tail = null, r.head = null, r.length = 0, t && "function" == typeof t.forEach) {
            t.forEach(function(e) {
                r.push(e);
            });
        } else if (arguments.length > 0) {
            for (var n = 0, o = arguments.length; n < o; n++) {
                r.push(arguments[n]);
            }
        }
        return r;
    }
    function t(e, t, r) {
        var n = t === e.head ? new o(r, null, t, e) : new o(r, t, t.next, e);
        return null === n.next && (e.tail = n), null === n.prev && (e.head = n), e.length++, 
        n;
    }
    function r(e, t) {
        e.tail = new o(t, e.tail, null, e), e.head || (e.head = e.tail), e.length++;
    }
    function n(e, t) {
        e.head = new o(t, null, e.head, e), e.tail || (e.tail = e.head), e.length++;
    }
    function o(e, t, r, n) {
        if (!(this instanceof o)) {
            return new o(e, t, r, n);
        }
        this.list = n, this.value = e, t ? (t.next = this, this.prev = t) : this.prev = null, 
        r ? (r.prev = this, this.next = r) : this.next = null;
    }
    al = 1, il = e, e.Node = o, e.create = e, e.prototype.removeNode = function(e) {
        if (e.list !== this) {
            throw new Error("removing node which does not belong to this list");
        }
        var t = e.next, r = e.prev;
        return t && (t.prev = r), r && (r.next = t), e === this.head && (this.head = t), 
        e === this.tail && (this.tail = r), e.list.length--, e.next = null, e.prev = null, 
        e.list = null, t;
    }, e.prototype.unshiftNode = function(e) {
        if (e !== this.head) {
            e.list && e.list.removeNode(e);
            var t = this.head;
            e.list = this, e.next = t, t && (t.prev = e), this.head = e, this.tail || (this.tail = e), 
            this.length++;
        }
    }, e.prototype.pushNode = function(e) {
        if (e !== this.tail) {
            e.list && e.list.removeNode(e);
            var t = this.tail;
            e.list = this, e.prev = t, t && (t.next = e), this.tail = e, this.head || (this.head = e), 
            this.length++;
        }
    }, e.prototype.push = function() {
        for (var e = 0, t = arguments.length; e < t; e++) {
            r(this, arguments[e]);
        }
        return this.length;
    }, e.prototype.unshift = function() {
        for (var e = 0, t = arguments.length; e < t; e++) {
            n(this, arguments[e]);
        }
        return this.length;
    }, e.prototype.pop = function() {
        if (this.tail) {
            var e = this.tail.value;
            return this.tail = this.tail.prev, this.tail ? this.tail.next = null : this.head = null, 
            this.length--, e;
        }
    }, e.prototype.shift = function() {
        if (this.head) {
            var e = this.head.value;
            return this.head = this.head.next, this.head ? this.head.prev = null : this.tail = null, 
            this.length--, e;
        }
    }, e.prototype.forEach = function(e, t) {
        t = t || this;
        for (var r = this.head, n = 0; null !== r; n++) {
            e.call(t, r.value, n, this), r = r.next;
        }
    }, e.prototype.forEachReverse = function(e, t) {
        t = t || this;
        for (var r = this.tail, n = this.length - 1; null !== r; n--) {
            e.call(t, r.value, n, this), r = r.prev;
        }
    }, e.prototype.get = function(e) {
        for (var t = 0, r = this.head; null !== r && t < e; t++) {
            r = r.next;
        }
        if (t === e && null !== r) {
            return r.value;
        }
    }, e.prototype.getReverse = function(e) {
        for (var t = 0, r = this.tail; null !== r && t < e; t++) {
            r = r.prev;
        }
        if (t === e && null !== r) {
            return r.value;
        }
    }, e.prototype.map = function(t, r) {
        r = r || this;
        for (var n = new e, o = this.head; null !== o; ) {
            n.push(t.call(r, o.value, this)), o = o.next;
        }
        return n;
    }, e.prototype.mapReverse = function(t, r) {
        r = r || this;
        for (var n = new e, o = this.tail; null !== o; ) {
            n.push(t.call(r, o.value, this)), o = o.prev;
        }
        return n;
    }, e.prototype.reduce = function(e, t) {
        var r, n = this.head;
        if (arguments.length > 1) {
            r = t;
        } else {
            if (!this.head) {
                throw new TypeError("Reduce of empty list with no initial value");
            }
            n = this.head.next, r = this.head.value;
        }
        for (var o = 0; null !== n; o++) {
            r = e(r, n.value, o), n = n.next;
        }
        return r;
    }, e.prototype.reduceReverse = function(e, t) {
        var r, n = this.tail;
        if (arguments.length > 1) {
            r = t;
        } else {
            if (!this.tail) {
                throw new TypeError("Reduce of empty list with no initial value");
            }
            n = this.tail.prev, r = this.tail.value;
        }
        for (var o = this.length - 1; null !== n; o--) {
            r = e(r, n.value, o), n = n.prev;
        }
        return r;
    }, e.prototype.toArray = function() {
        for (var e = new Array(this.length), t = 0, r = this.head; null !== r; t++) {
            e[t] = r.value, r = r.next;
        }
        return e;
    }, e.prototype.toArrayReverse = function() {
        for (var e = new Array(this.length), t = 0, r = this.tail; null !== r; t++) {
            e[t] = r.value, r = r.prev;
        }
        return e;
    }, e.prototype.slice = function(t, r) {
        (r = r || this.length) < 0 && (r += this.length), (t = t || 0) < 0 && (t += this.length);
        var n = new e;
        if (r < t || r < 0) {
            return n;
        }
        t < 0 && (t = 0), r > this.length && (r = this.length);
        for (var o = 0, i = this.head; null !== i && o < t; o++) {
            i = i.next;
        }
        for (;null !== i && o < r; o++, i = i.next) {
            n.push(i.value);
        }
        return n;
    }, e.prototype.sliceReverse = function(t, r) {
        (r = r || this.length) < 0 && (r += this.length), (t = t || 0) < 0 && (t += this.length);
        var n = new e;
        if (r < t || r < 0) {
            return n;
        }
        t < 0 && (t = 0), r > this.length && (r = this.length);
        for (var o = this.length, i = this.tail; null !== i && o > r; o--) {
            i = i.prev;
        }
        for (;null !== i && o > t; o--, i = i.prev) {
            n.push(i.value);
        }
        return n;
    }, e.prototype.splice = function(e, r, ...n) {
        e > this.length && (e = this.length - 1), e < 0 && (e = this.length + e);
        for (var o = 0, i = this.head; null !== i && o < e; o++) {
            i = i.next;
        }
        var a = [];
        for (o = 0; i && o < r; o++) {
            a.push(i.value), i = this.removeNode(i);
        }
        null === i && (i = this.tail), i !== this.head && i !== this.tail && (i = i.prev);
        for (o = 0; o < n.length; o++) {
            i = t(this, i, n[o]);
        }
        return a;
    }, e.prototype.reverse = function() {
        for (var e = this.head, t = this.tail, r = e; null !== r; r = r.prev) {
            var n = r.prev;
            r.prev = r.next, r.next = n;
        }
        return this.head = t, this.tail = e, this;
    };
    try {
        (ol ? nl : (ol = 1, nl = function(e) {
            e.prototype[Symbol.iterator] = function*() {
                for (let e = this.head; e; e = e.next) {
                    yield e.value;
                }
            };
        }))(e);
    } catch (e) {}
    return il;
}

function vl() {
    if (cl) {
        return ll;
    }
    cl = 1;
    class e {
        constructor(t, r) {
            if (r = n(r), t instanceof e) {
                return t.loose === !!r.loose && t.includePrerelease === !!r.includePrerelease ? t : new e(t.raw, r);
            }
            if (t instanceof o) {
                return this.raw = t.value, this.set = [ [ t ] ], this.format(), this;
            }
            if (this.options = r, this.loose = !!r.loose, this.includePrerelease = !!r.includePrerelease, 
            this.raw = t.trim().split(/\s+/).join(" "), this.set = this.raw.split("||").map(e => this.parseRange(e.trim())).filter(e => e.length), 
            !this.set.length) {
                throw new TypeError(`Invalid SemVer Range: ${this.raw}`);
            }
            if (this.set.length > 1) {
                const e = this.set[0];
                if (this.set = this.set.filter(e => !h(e[0])), 0 === this.set.length) {
                    this.set = [ e ];
                } else if (this.set.length > 1) {
                    for (const e of this.set) {
                        if (1 === e.length && v(e[0])) {
                            this.set = [ e ];
                            break;
                        }
                    }
                }
            }
            this.format();
        }
        format() {
            return this.range = this.set.map(e => e.join(" ").trim()).join("||").trim(), this.range;
        }
        toString() {
            return this.range;
        }
        parseRange(e) {
            const t = ((this.options.includePrerelease && d) | (this.options.loose && p)) + ":" + e, n = r.get(t);
            if (n) {
                return n;
            }
            const a = this.options.loose, v = a ? s[u.HYPHENRANGELOOSE] : s[u.HYPHENRANGE];
            e = e.replace(v, C(this.options.includePrerelease)), i("hyphen replace", e), e = e.replace(s[u.COMPARATORTRIM], l), 
            i("comparator trim", e), e = e.replace(s[u.TILDETRIM], c), i("tilde trim", e), e = e.replace(s[u.CARETTRIM], f), 
            i("caret trim", e);
            let g = e.split(" ").map(e => m(e, this.options)).join(" ").split(/\s+/).map(e => O(e, this.options));
            a && (g = g.filter(e => (i("loose invalid filter", e, this.options), !!e.match(s[u.COMPARATORLOOSE])))), 
            i("range list", g);
            const y = new Map, _ = g.map(e => new o(e, this.options));
            for (const e of _) {
                if (h(e)) {
                    return [ e ];
                }
                y.set(e.value, e);
            }
            y.size > 1 && y.has("") && y.delete("");
            const E = [ ...y.values() ];
            return r.set(t, E), E;
        }
        intersects(t, r) {
            if (!(t instanceof e)) {
                throw new TypeError("a Range is required");
            }
            return this.set.some(e => g(e, r) && t.set.some(t => g(t, r) && e.every(e => t.every(t => e.intersects(t, r)))));
        }
        test(e) {
            if (!e) {
                return !1;
            }
            if ("string" == typeof e) {
                try {
                    e = new a(e, this.options);
                } catch (e) {
                    return !1;
                }
            }
            for (let t = 0; t < this.set.length; t++) {
                if (x(this.set[t], e, this.options)) {
                    return !0;
                }
            }
            return !1;
        }
    }
    ll = e;
    const t = function() {
        if (ul) {
            return sl;
        }
        ul = 1;
        const e = hl(), t = Symbol("max"), r = Symbol("length"), n = Symbol("lengthCalculator"), o = Symbol("allowStale"), i = Symbol("maxAge"), a = Symbol("dispose"), s = Symbol("noDisposeOnSet"), u = Symbol("lruList"), l = Symbol("cache"), c = Symbol("updateAgeOnGet"), f = () => 1, d = (e, t, r) => {
            const n = e[l].get(t);
            if (n) {
                const t = n.value;
                if (p(e, t)) {
                    if (v(e, n), !e[o]) {
                        return;
                    }
                } else {
                    r && (e[c] && (n.value.now = Date.now()), e[u].unshiftNode(n));
                }
                return t.value;
            }
        }, p = (e, t) => {
            if (!t || !t.maxAge && !e[i]) {
                return !1;
            }
            const r = Date.now() - t.now;
            return t.maxAge ? r > t.maxAge : e[i] && r > e[i];
        }, h = e => {
            if (e[r] > e[t]) {
                for (let n = e[u].tail; e[r] > e[t] && null !== n; ) {
                    const t = n.prev;
                    v(e, n), n = t;
                }
            }
        }, v = (e, t) => {
            if (t) {
                const n = t.value;
                e[a] && e[a](n.key, n.value), e[r] -= n.length, e[l].delete(n.key), e[u].removeNode(t);
            }
        };
        class g {
            constructor(e, t, r, n, o) {
                this.key = e, this.value = t, this.length = r, this.now = n, this.maxAge = o || 0;
            }
        }
        const m = (e, t, r, n) => {
            let i = r.value;
            p(e, i) && (v(e, r), e[o] || (i = void 0)), i && t.call(n, i.value, i.key, e);
        };
        return sl = class {
            constructor(e) {
                if ("number" == typeof e && (e = {
                    max: e
                }), e || (e = {}), e.max && ("number" != typeof e.max || e.max < 0)) {
                    throw new TypeError("max must be a non-negative number");
                }
                this[t] = e.max || 1 / 0;
                const r = e.length || f;
                if (this[n] = "function" != typeof r ? f : r, this[o] = e.stale || !1, e.maxAge && "number" != typeof e.maxAge) {
                    throw new TypeError("maxAge must be a number");
                }
                this[i] = e.maxAge || 0, this[a] = e.dispose, this[s] = e.noDisposeOnSet || !1, 
                this[c] = e.updateAgeOnGet || !1, this.reset();
            }
            set max(e) {
                if ("number" != typeof e || e < 0) {
                    throw new TypeError("max must be a non-negative number");
                }
                this[t] = e || 1 / 0, h(this);
            }
            get max() {
                return this[t];
            }
            set allowStale(e) {
                this[o] = !!e;
            }
            get allowStale() {
                return this[o];
            }
            set maxAge(e) {
                if ("number" != typeof e) {
                    throw new TypeError("maxAge must be a non-negative number");
                }
                this[i] = e, h(this);
            }
            get maxAge() {
                return this[i];
            }
            set lengthCalculator(e) {
                "function" != typeof e && (e = f), e !== this[n] && (this[n] = e, this[r] = 0, this[u].forEach(e => {
                    e.length = this[n](e.value, e.key), this[r] += e.length;
                })), h(this);
            }
            get lengthCalculator() {
                return this[n];
            }
            get length() {
                return this[r];
            }
            get itemCount() {
                return this[u].length;
            }
            rforEach(e, t) {
                t = t || this;
                for (let r = this[u].tail; null !== r; ) {
                    const n = r.prev;
                    m(this, e, r, t), r = n;
                }
            }
            forEach(e, t) {
                t = t || this;
                for (let r = this[u].head; null !== r; ) {
                    const n = r.next;
                    m(this, e, r, t), r = n;
                }
            }
            keys() {
                return this[u].toArray().map(e => e.key);
            }
            values() {
                return this[u].toArray().map(e => e.value);
            }
            reset() {
                this[a] && this[u] && this[u].length && this[u].forEach(e => this[a](e.key, e.value)), 
                this[l] = new Map, this[u] = new e, this[r] = 0;
            }
            dump() {
                return this[u].map(e => !p(this, e) && {
                    k: e.key,
                    v: e.value,
                    e: e.now + (e.maxAge || 0)
                }).toArray().filter(e => e);
            }
            dumpLru() {
                return this[u];
            }
            set(e, o, c) {
                if ((c = c || this[i]) && "number" != typeof c) {
                    throw new TypeError("maxAge must be a number");
                }
                const f = c ? Date.now() : 0, d = this[n](o, e);
                if (this[l].has(e)) {
                    if (d > this[t]) {
                        return v(this, this[l].get(e)), !1;
                    }
                    const n = this[l].get(e).value;
                    return this[a] && (this[s] || this[a](e, n.value)), n.now = f, n.maxAge = c, n.value = o, 
                    this[r] += d - n.length, n.length = d, this.get(e), h(this), !0;
                }
                const p = new g(e, o, d, f, c);
                return p.length > this[t] ? (this[a] && this[a](e, o), !1) : (this[r] += p.length, 
                this[u].unshift(p), this[l].set(e, this[u].head), h(this), !0);
            }
            has(e) {
                if (!this[l].has(e)) {
                    return !1;
                }
                const t = this[l].get(e).value;
                return !p(this, t);
            }
            get(e) {
                return d(this, e, !0);
            }
            peek(e) {
                return d(this, e, !1);
            }
            pop() {
                const e = this[u].tail;
                return e ? (v(this, e), e.value) : null;
            }
            del(e) {
                v(this, this[l].get(e));
            }
            load(e) {
                this.reset();
                const t = Date.now();
                for (let r = e.length - 1; r >= 0; r--) {
                    const n = e[r], o = n.e || 0;
                    if (0 === o) {
                        this.set(n.k, n.v);
                    } else {
                        const e = o - t;
                        e > 0 && this.set(n.k, n.v, e);
                    }
                }
            }
            prune() {
                this[l].forEach((e, t) => d(this, t, !1));
            }
        }, sl;
    }(), r = new t({
        max: 1e3
    }), n = Vs, o = gl(), i = Hs, a = nu, {safeRe: s, t: u, comparatorTrimReplace: l, tildeTrimReplace: c, caretTrimReplace: f} = $s, {FLAG_INCLUDE_PRERELEASE: d, FLAG_LOOSE: p} = zs, h = e => "<0.0.0-0" === e.value, v = e => "" === e.value, g = (e, t) => {
        let r = !0;
        const n = e.slice();
        let o = n.pop();
        for (;r && n.length; ) {
            r = n.every(e => o.intersects(e, t)), o = n.pop();
        }
        return r;
    }, m = (e, t) => (i("comp", e, t), e = b(e, t), i("caret", e), e = _(e, t), i("tildes", e), 
    e = D(e, t), i("xrange", e), e = A(e, t), i("stars", e), e), y = e => !e || "x" === e.toLowerCase() || "*" === e, _ = (e, t) => e.trim().split(/\s+/).map(e => E(e, t)).join(" "), E = (e, t) => {
        const r = t.loose ? s[u.TILDELOOSE] : s[u.TILDE];
        return e.replace(r, (t, r, n, o, a) => {
            let s;
            return i("tilde", e, t, r, n, o, a), y(r) ? s = "" : y(n) ? s = `>=${r}.0.0 <${+r + 1}.0.0-0` : y(o) ? s = `>=${r}.${n}.0 <${r}.${+n + 1}.0-0` : a ? (i("replaceTilde pr", a), 
            s = `>=${r}.${n}.${o}-${a} <${r}.${+n + 1}.0-0`) : s = `>=${r}.${n}.${o} <${r}.${+n + 1}.0-0`, 
            i("tilde return", s), s;
        });
    }, b = (e, t) => e.trim().split(/\s+/).map(e => w(e, t)).join(" "), w = (e, t) => {
        i("caret", e, t);
        const r = t.loose ? s[u.CARETLOOSE] : s[u.CARET], n = t.includePrerelease ? "-0" : "";
        return e.replace(r, (t, r, o, a, s) => {
            let u;
            return i("caret", e, t, r, o, a, s), y(r) ? u = "" : y(o) ? u = `>=${r}.0.0${n} <${+r + 1}.0.0-0` : y(a) ? u = "0" === r ? `>=${r}.${o}.0${n} <${r}.${+o + 1}.0-0` : `>=${r}.${o}.0${n} <${+r + 1}.0.0-0` : s ? (i("replaceCaret pr", s), 
            u = "0" === r ? "0" === o ? `>=${r}.${o}.${a}-${s} <${r}.${o}.${+a + 1}-0` : `>=${r}.${o}.${a}-${s} <${r}.${+o + 1}.0-0` : `>=${r}.${o}.${a}-${s} <${+r + 1}.0.0-0`) : (i("no pr"), 
            u = "0" === r ? "0" === o ? `>=${r}.${o}.${a}${n} <${r}.${o}.${+a + 1}-0` : `>=${r}.${o}.${a}${n} <${r}.${+o + 1}.0-0` : `>=${r}.${o}.${a} <${+r + 1}.0.0-0`), 
            i("caret return", u), u;
        });
    }, D = (e, t) => (i("replaceXRanges", e, t), e.split(/\s+/).map(e => S(e, t)).join(" ")), S = (e, t) => {
        e = e.trim();
        const r = t.loose ? s[u.XRANGELOOSE] : s[u.XRANGE];
        return e.replace(r, (r, n, o, a, s, u) => {
            i("xRange", e, r, n, o, a, s, u);
            const l = y(o), c = l || y(a), f = c || y(s), d = f;
            return "=" === n && d && (n = ""), u = t.includePrerelease ? "-0" : "", l ? r = ">" === n || "<" === n ? "<0.0.0-0" : "*" : n && d ? (c && (a = 0), 
            s = 0, ">" === n ? (n = ">=", c ? (o = +o + 1, a = 0, s = 0) : (a = +a + 1, s = 0)) : "<=" === n && (n = "<", 
            c ? o = +o + 1 : a = +a + 1), "<" === n && (u = "-0"), r = `${n + o}.${a}.${s}${u}`) : c ? r = `>=${o}.0.0${u} <${+o + 1}.0.0-0` : f && (r = `>=${o}.${a}.0${u} <${o}.${+a + 1}.0-0`), 
            i("xRange return", r), r;
        });
    }, A = (e, t) => (i("replaceStars", e, t), e.trim().replace(s[u.STAR], "")), O = (e, t) => (i("replaceGTE0", e, t), 
    e.trim().replace(s[t.includePrerelease ? u.GTE0PRE : u.GTE0], "")), C = e => (t, r, n, o, i, a, s, u, l, c, f, d, p) => `${r = y(n) ? "" : y(o) ? `>=${n}.0.0${e ? "-0" : ""}` : y(i) ? `>=${n}.${o}.0${e ? "-0" : ""}` : a ? `>=${r}` : `>=${r}${e ? "-0" : ""}`} ${u = y(l) ? "" : y(c) ? `<${+l + 1}.0.0-0` : y(f) ? `<${l}.${+c + 1}.0-0` : d ? `<=${l}.${c}.${f}-${d}` : e ? `<${l}.${c}.${+f + 1}-0` : `<=${u}`}`.trim(), x = (e, t, r) => {
        for (let r = 0; r < e.length; r++) {
            if (!e[r].test(t)) {
                return !1;
            }
        }
        if (t.prerelease.length && !r.includePrerelease) {
            for (let r = 0; r < e.length; r++) {
                if (i(e[r].semver), e[r].semver !== o.ANY && e[r].semver.prerelease.length > 0) {
                    const n = e[r].semver;
                    if (n.major === t.major && n.minor === t.minor && n.patch === t.patch) {
                        return !0;
                    }
                }
            }
            return !1;
        }
        return !0;
    };
    return ll;
}

function gl() {
    if (dl) {
        return fl;
    }
    dl = 1;
    const e = Symbol("SemVer ANY");
    class t {
        static get ANY() {
            return e;
        }
        constructor(n, o) {
            if (o = r(o), n instanceof t) {
                if (n.loose === !!o.loose) {
                    return n;
                }
                n = n.value;
            }
            n = n.trim().split(/\s+/).join(" "), a("comparator", n, o), this.options = o, this.loose = !!o.loose, 
            this.parse(n), this.semver === e ? this.value = "" : this.value = this.operator + this.semver.version, 
            a("comp", this);
        }
        parse(t) {
            const r = this.options.loose ? n[o.COMPARATORLOOSE] : n[o.COMPARATOR], i = t.match(r);
            if (!i) {
                throw new TypeError(`Invalid comparator: ${t}`);
            }
            this.operator = void 0 !== i[1] ? i[1] : "", "=" === this.operator && (this.operator = ""), 
            i[2] ? this.semver = new s(i[2], this.options.loose) : this.semver = e;
        }
        toString() {
            return this.value;
        }
        test(t) {
            if (a("Comparator.test", t, this.options.loose), this.semver === e || t === e) {
                return !0;
            }
            if ("string" == typeof t) {
                try {
                    t = new s(t, this.options);
                } catch (e) {
                    return !1;
                }
            }
            return i(t, this.operator, this.semver, this.options);
        }
        intersects(e, n) {
            if (!(e instanceof t)) {
                throw new TypeError("a Comparator is required");
            }
            return "" === this.operator ? "" === this.value || new u(e.value, n).test(this.value) : "" === e.operator ? "" === e.value || new u(this.value, n).test(e.semver) : (!(n = r(n)).includePrerelease || "<0.0.0-0" !== this.value && "<0.0.0-0" !== e.value) && (!(!n.includePrerelease && (this.value.startsWith("<0.0.0") || e.value.startsWith("<0.0.0"))) && (!(!this.operator.startsWith(">") || !e.operator.startsWith(">")) || (!(!this.operator.startsWith("<") || !e.operator.startsWith("<")) || (!(this.semver.version !== e.semver.version || !this.operator.includes("=") || !e.operator.includes("=")) || (!!(i(this.semver, "<", e.semver, n) && this.operator.startsWith(">") && e.operator.startsWith("<")) || !!(i(this.semver, ">", e.semver, n) && this.operator.startsWith("<") && e.operator.startsWith(">")))))));
        }
    }
    fl = t;
    const r = Vs, {safeRe: n, t: o} = $s, i = Yu, a = Hs, s = nu, u = vl();
    return fl;
}

const ml = vl();

var yl = (e, t, r) => {
    try {
        t = new ml(t, r);
    } catch (e) {
        return !1;
    }
    return t.test(e);
};

const _l = vl();

var El = (e, t) => new _l(e, t).set.map(e => e.map(e => e.value).join(" ").trim().split(" "));

const bl = nu, wl = vl();

var Dl = (e, t, r) => {
    let n = null, o = null, i = null;
    try {
        i = new wl(t, r);
    } catch (e) {
        return null;
    }
    return e.forEach(e => {
        i.test(e) && (n && -1 !== o.compare(e) || (n = e, o = new bl(n, r)));
    }), n;
};

const Sl = nu, Al = vl();

var Ol = (e, t, r) => {
    let n = null, o = null, i = null;
    try {
        i = new Al(t, r);
    } catch (e) {
        return null;
    }
    return e.forEach(e => {
        i.test(e) && (n && 1 !== o.compare(e) || (n = e, o = new Sl(n, r)));
    }), n;
};

const Cl = nu, xl = vl(), Fl = Tu;

var Ml = (e, t) => {
    e = new xl(e, t);
    let r = new Cl("0.0.0");
    if (e.test(r)) {
        return r;
    }
    if (r = new Cl("0.0.0-0"), e.test(r)) {
        return r;
    }
    r = null;
    for (let t = 0; t < e.set.length; ++t) {
        const n = e.set[t];
        let o = null;
        n.forEach(e => {
            const t = new Cl(e.semver.version);
            switch (e.operator) {
              case ">":
                0 === t.prerelease.length ? t.patch++ : t.prerelease.push(0), t.raw = t.format();

              case "":
              case ">=":
                o && !Fl(t, o) || (o = t);
                break;

              case "<":
              case "<=":
                break;

              default:
                throw new Error(`Unexpected operation: ${e.operator}`);
            }
        }), !o || r && !Fl(r, o) || (r = o);
    }
    return r && e.test(r) ? r : null;
};

const Pl = vl();

var Il = (e, t) => {
    try {
        return new Pl(e, t).range || "*";
    } catch (e) {
        return null;
    }
};

const kl = nu, Rl = gl(), {ANY: Tl} = Rl, jl = vl(), Ll = yl, Nl = Tu, Bl = Lu, Ul = Wu, zl = $u;

var Hl = (e, t, r, n) => {
    let o, i, a, s, u;
    switch (e = new kl(e, n), t = new jl(t, n), r) {
      case ">":
        o = Nl, i = Ul, a = Bl, s = ">", u = ">=";
        break;

      case "<":
        o = Bl, i = zl, a = Nl, s = "<", u = "<=";
        break;

      default:
        throw new TypeError('Must provide a hilo val of "<" or ">"');
    }
    if (Ll(e, t, n)) {
        return !1;
    }
    for (let r = 0; r < t.set.length; ++r) {
        const l = t.set[r];
        let c = null, f = null;
        if (l.forEach(e => {
            e.semver === Tl && (e = new Rl(">=0.0.0")), c = c || e, f = f || e, o(e.semver, c.semver, n) ? c = e : a(e.semver, f.semver, n) && (f = e);
        }), c.operator === s || c.operator === u) {
            return !1;
        }
        if ((!f.operator || f.operator === s) && i(e, f.semver)) {
            return !1;
        }
        if (f.operator === u && a(e, f.semver)) {
            return !1;
        }
    }
    return !0;
};

const $l = Hl;

var Gl = (e, t, r) => $l(e, t, ">", r);

const Wl = Hl;

var Vl = (e, t, r) => Wl(e, t, "<", r);

const Kl = vl();

var ql = (e, t, r) => (e = new Kl(e, r), t = new Kl(t, r), e.intersects(t, r));

const Jl = yl, Xl = Du;

const Zl = vl(), Yl = gl(), {ANY: Ql} = Yl, ec = yl, tc = Du, rc = [ new Yl(">=0.0.0-0") ], nc = [ new Yl(">=0.0.0") ], oc = (e, t, r) => {
    if (e === t) {
        return !0;
    }
    if (1 === e.length && e[0].semver === Ql) {
        if (1 === t.length && t[0].semver === Ql) {
            return !0;
        }
        e = r.includePrerelease ? rc : nc;
    }
    if (1 === t.length && t[0].semver === Ql) {
        if (r.includePrerelease) {
            return !0;
        }
        t = nc;
    }
    const n = new Set;
    let o, i, a, s, u, l, c;
    for (const t of e) {
        ">" === t.operator || ">=" === t.operator ? o = ic(o, t, r) : "<" === t.operator || "<=" === t.operator ? i = ac(i, t, r) : n.add(t.semver);
    }
    if (n.size > 1) {
        return null;
    }
    if (o && i) {
        if (a = tc(o.semver, i.semver, r), a > 0) {
            return null;
        }
        if (0 === a && (">=" !== o.operator || "<=" !== i.operator)) {
            return null;
        }
    }
    for (const e of n) {
        if (o && !ec(e, String(o), r)) {
            return null;
        }
        if (i && !ec(e, String(i), r)) {
            return null;
        }
        for (const n of t) {
            if (!ec(e, String(n), r)) {
                return !1;
            }
        }
        return !0;
    }
    let f = !(!i || r.includePrerelease || !i.semver.prerelease.length) && i.semver, d = !(!o || r.includePrerelease || !o.semver.prerelease.length) && o.semver;
    f && 1 === f.prerelease.length && "<" === i.operator && 0 === f.prerelease[0] && (f = !1);
    for (const e of t) {
        if (c = c || ">" === e.operator || ">=" === e.operator, l = l || "<" === e.operator || "<=" === e.operator, 
        o) {
            if (d && e.semver.prerelease && e.semver.prerelease.length && e.semver.major === d.major && e.semver.minor === d.minor && e.semver.patch === d.patch && (d = !1), 
            ">" === e.operator || ">=" === e.operator) {
                if (s = ic(o, e, r), s === e && s !== o) {
                    return !1;
                }
            } else if (">=" === o.operator && !ec(o.semver, String(e), r)) {
                return !1;
            }
        }
        if (i) {
            if (f && e.semver.prerelease && e.semver.prerelease.length && e.semver.major === f.major && e.semver.minor === f.minor && e.semver.patch === f.patch && (f = !1), 
            "<" === e.operator || "<=" === e.operator) {
                if (u = ac(i, e, r), u === e && u !== i) {
                    return !1;
                }
            } else if ("<=" === i.operator && !ec(i.semver, String(e), r)) {
                return !1;
            }
        }
        if (!e.operator && (i || o) && 0 !== a) {
            return !1;
        }
    }
    return !(o && l && !i && 0 !== a) && (!(i && c && !o && 0 !== a) && (!d && !f));
}, ic = (e, t, r) => {
    if (!e) {
        return t;
    }
    const n = tc(e.semver, t.semver, r);
    return n > 0 ? e : n < 0 || ">" === t.operator && ">=" === e.operator ? t : e;
}, ac = (e, t, r) => {
    if (!e) {
        return t;
    }
    const n = tc(e.semver, t.semver, r);
    return n < 0 ? e : n > 0 || "<" === t.operator && "<=" === e.operator ? t : e;
};

var sc = (e, t, r = {}) => {
    if (e === t) {
        return !0;
    }
    e = new Zl(e, r), t = new Zl(t, r);
    let n = !1;
    e: for (const o of e.set) {
        for (const e of t.set) {
            const t = oc(o, e, r);
            if (n = n || null !== t, t) {
                continue e;
            }
        }
        if (n) {
            return !1;
        }
    }
    return !0;
};

const uc = $s, lc = zs, cc = nu, fc = Js, dc = (e, t, r) => {
    const n = [];
    let o = null, i = null;
    const a = e.sort((e, t) => Xl(e, t, r));
    for (const e of a) {
        Jl(e, t, r) ? (i = e, o || (o = e)) : (i && n.push([ o, i ]), i = null, o = null);
    }
    o && n.push([ o, null ]);
    const s = [];
    for (const [e, t] of n) {
        e === t ? s.push(e) : t || e !== a[0] ? t ? e === a[0] ? s.push(`<=${t}`) : s.push(`${e} - ${t}`) : s.push(`>=${e}`) : s.push("*");
    }
    const u = s.join(" || "), l = "string" == typeof t.raw ? t.raw : String(t);
    return u.length < l.length ? u : t;
};

var pc = {
    parse: iu,
    valid: su,
    clean: lu,
    inc: fu,
    diff: pu,
    major: vu,
    minor: mu,
    patch: _u,
    prerelease: bu,
    compare: Du,
    rcompare: Au,
    compareLoose: Cu,
    compareBuild: Fu,
    sort: Pu,
    rsort: ku,
    gt: Tu,
    lt: Lu,
    eq: Bu,
    neq: zu,
    gte: $u,
    lte: Wu,
    cmp: Yu,
    coerce: pl,
    Comparator: gl(),
    Range: vl(),
    satisfies: yl,
    toComparators: El,
    maxSatisfying: Dl,
    minSatisfying: Ol,
    minVersion: Ml,
    validRange: Il,
    outside: Hl,
    gtr: Gl,
    ltr: Vl,
    intersects: ql,
    simplifyRange: dc,
    subset: sc,
    SemVer: cc,
    re: uc.re,
    src: uc.src,
    tokens: uc.t,
    SEMVER_SPEC_VERSION: lc.SEMVER_SPEC_VERSION,
    RELEASE_TYPES: lc.RELEASE_TYPES,
    compareIdentifiers: fc.compareIdentifiers,
    rcompareIdentifiers: fc.rcompareIdentifiers
}, hc = {}, vc = {}, gc = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(vc, "__esModule", {
    value: !0
}), vc.logFormatedErrorAndExit = vc.logFormatedError = vc.logError = vc.logInfo = vc.logErrorAndExit = void 0;

const mc = gc(a);

function yc(e, t, r) {
    let n = "";
    const o = "[31m", i = function(e) {
        if (!e || e.length < 5) {
            return;
        }
        return e.slice(3, 5);
    }(e);
    let a = "";
    if (i) {
        const e = Ec[i];
        a = null != e ? e : a;
    }
    n += `> hvigor ${o}ERROR: ${e} ${a}`, n += `${o}${mc.default.EOL}Error Message: ${t}`, 
    n += `${o}${mc.default.EOL}${mc.default.EOL}* Try the following: `, r.forEach(e => {
        n += `${o}${mc.default.EOL}  > ${e}[39m`;
    }), console.error(n);
}

var _c;

vc.logErrorAndExit = function(e) {
    e instanceof Error ? console.error(e.message) : console.error(e), process.exit(-1);
}, vc.logInfo = function(e) {
    console.log(e);
}, vc.logError = function(e) {
    console.error(e);
}, vc.logFormatedError = yc, vc.logFormatedErrorAndExit = function(e, t, r) {
    yc(e, t, r), process.exit(-1);
}, function(e) {
    e.ERROR_00 = "00", e.ERROR_01 = "01", e.ERROR_02 = "02", e.ERROR_03 = "03", e.ERROR_04 = "04", 
    e.ERROR_05 = "05", e.ERROR_06 = "06", e.ERROR_07 = "07", e.ERROR_08 = "08";
}(_c || (_c = {}));

const Ec = {
    [_c.ERROR_00]: "Unknown Error",
    [_c.ERROR_01]: "Dependency Error",
    [_c.ERROR_02]: "Script Error",
    [_c.ERROR_03]: "Configuration Error",
    [_c.ERROR_04]: "Not Found",
    [_c.ERROR_05]: "Syntax Error",
    [_c.ERROR_06]: "Specification Limit Violation",
    [_c.ERROR_07]: "Permissions Error",
    [_c.ERROR_08]: "Operation Error"
};

var bc = {}, wc = {}, Dc = {}, Sc = {}, Ac = {};

!function(e) {
    var t;
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.ToolErrorCode = e.ErrorOwnerShip = void 0, (t = e.ErrorOwnerShip || (e.ErrorOwnerShip = {})).EOS_0 = "0", 
    t.EOS_1 = "1", t.EOS_2 = "2", t.EOS_3 = "3", t.EOS_4 = "4", t.EOS_5 = "5", t.EOS_6 = "6", 
    t.EOS_7 = "7", function(e) {
        e.TEC_00 = "00", e.TEC_10 = "10", e.TEC_11 = "11", e.TEC_12 = "12", e.TEC_21 = "21", 
        e.TEC_22 = "22", e.TEC_23 = "23", e.TEC_24 = "24";
    }(e.ToolErrorCode || (e.ToolErrorCode = {}));
}(Ac);

var Oc = {}, Cc = {}, xc = {};

!function(e) {
    var n = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.getUrl = e.DEVELOPER_URL = void 0;
    const o = n(t), i = n(r);
    e.DEVELOPER_URL = "DEVELOPER_URL";
    const a = i.default.resolve(__dirname, "../../res/config.json");
    e.getUrl = function(e) {
        const t = function(e, t = "utf-8") {
            if (o.default.existsSync(e)) {
                try {
                    const r = o.default.readFileSync(e, {
                        encoding: t
                    });
                    return JSON.parse(r);
                } catch (e) {
                    return;
                }
            }
        }(a);
        if (t && t[e]) {
            return t[e];
        }
    };
}(xc), function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.ErrorCodeDescription = e.ErrorCode = e.MetricLogType = e.LogLevel = e.SubSystemEnum = e.MergeKeyEnum = e.MergeType = e.CountryEnum = e.DEFAULT_MORE_INFO_URL_EN = e.DEFAULT_MORE_INFO_URL_CN = e.SPLIT_TAG = e.NORMAL_LOG_TYPE = e.PLUGIN_LOG_TYPE = e.HVIGOR_USER_HOME_DIR_NAME = e.BUILD_CACHE_DIR = e.UNDEFINED_POS = e.UNDEFINED_CAUSE = e.UNDEFINED_DESC = e.UNDEFINED_CODE = void 0;
    const t = xc;
    var r, n;
    e.UNDEFINED_CODE = "00000000", e.UNDEFINED_DESC = "", e.UNDEFINED_CAUSE = "Unknown", 
    e.UNDEFINED_POS = "", e.BUILD_CACHE_DIR = "build-cache-dir", e.HVIGOR_USER_HOME_DIR_NAME = ".hvigor", 
    e.PLUGIN_LOG_TYPE = "plugin_log", e.NORMAL_LOG_TYPE = "normal_log", e.SPLIT_TAG = "<HVIGOR_ERROR_SPLIT>", 
    e.DEFAULT_MORE_INFO_URL_CN = (0, t.getUrl)(t.DEVELOPER_URL) ? `${(0, t.getUrl)(t.DEVELOPER_URL)}/consumer/cn/customerService` : void 0, 
    e.DEFAULT_MORE_INFO_URL_EN = (0, t.getUrl)(t.DEVELOPER_URL) ? `${(0, t.getUrl)(t.DEVELOPER_URL)}/consumer/en/customerService` : void 0, 
    (n = e.CountryEnum || (e.CountryEnum = {})).CN = "cn", n.EN = "en", function(e) {
        e[e.COLLECT_LAST = 1] = "COLLECT_LAST", e[e.COLLECT_FIRST = 2] = "COLLECT_FIRST", 
        e[e.COLLECT_ALL = 3] = "COLLECT_ALL";
    }(e.MergeType || (e.MergeType = {})), function(e) {
        e.CODE = "code", e.CAUSE = "cause", e.POSITION = "position", e.SOLUTIONS = "solutions", 
        e.MORE_INFO = "moreInfo";
    }(e.MergeKeyEnum || (e.MergeKeyEnum = {})), function(e) {
        e.UNKNOWN = "000", e.HVIGOR = "103", e.PACKAGE = "200";
    }(e.SubSystemEnum || (e.SubSystemEnum = {})), function(e) {
        e.DEBUG = "debug", e.INFO = "info", e.WARN = "warn", e.ERROR = "error";
    }(e.LogLevel || (e.LogLevel = {})), function(e) {
        e.DEBUG = "debug", e.INFO = "info", e.WARN = "warn", e.ERROR = "error", e.DETAIL = "detail";
    }(e.MetricLogType || (e.MetricLogType = {})), function(e) {
        e.ERROR_00 = "00", e.ERROR_01 = "01", e.ERROR_02 = "02", e.ERROR_03 = "03", e.ERROR_04 = "04", 
        e.ERROR_05 = "05", e.ERROR_06 = "06", e.ERROR_07 = "07", e.ERROR_08 = "08", e.ERROR_09 = "09";
    }(r = e.ErrorCode || (e.ErrorCode = {})), e.ErrorCodeDescription = {
        [r.ERROR_00]: "Unknown Error",
        [r.ERROR_01]: "Dependency Error",
        [r.ERROR_02]: "Script Error",
        [r.ERROR_03]: "Configuration Error",
        [r.ERROR_04]: "Not Found",
        [r.ERROR_05]: "Syntax Error",
        [r.ERROR_06]: "Specification Limit Violation",
        [r.ERROR_07]: "Permissions Error",
        [r.ERROR_08]: "Operation Error",
        [r.ERROR_09]: "ArkTS Compiler Error"
    };
}(Cc);

var Fc = {}, Mc = {};

!function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.AdaptorError = void 0;
    class t extends Error {
        constructor(e, r) {
            super(e), this.name = t.NAME, r && (this.stack = r);
        }
    }
    e.AdaptorError = t, t.NAME = "AdaptorError";
}(Mc);

var Pc = {}, Ic = {};

!function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.HvigorLogInfo = void 0;
    const t = Cc;
    e.HvigorLogInfo = class {
        constructor(e) {
            this.message = "", this.logLevel = t.LogLevel.DEBUG, e && (this.message = e.message, 
            this.logLevel = e.logLevel);
        }
        setMessage(e) {
            return this.message = e, this;
        }
        setLogLevel(e) {
            return this.logLevel = e, this;
        }
        getMessage() {
            return this.message;
        }
        getLogLevel() {
            return this.logLevel;
        }
    };
}(Ic), function(e) {
    var t = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.HvigorErrorCommonAdapter = void 0;
    const r = t(a), n = y, o = Ic, i = Cc;
    class s {
        convertErrorToLogInfo(e) {
            return new o.HvigorLogInfo({
                message: s.red + this.combinePhase(e) + s.reset,
                logLevel: i.LogLevel.ERROR
            });
        }
        convertWarnToLogInfo(e) {
            return new o.HvigorLogInfo({
                message: s.yellow + e + s.reset,
                logLevel: i.LogLevel.WARN
            });
        }
        convertInfoToLogInfo(e) {
            return new o.HvigorLogInfo({
                message: s.green + e + s.reset,
                logLevel: i.LogLevel.INFO
            });
        }
        convertDebugToLogInfo(e) {
            return new o.HvigorLogInfo({
                message: e,
                logLevel: i.LogLevel.DEBUG
            });
        }
        combinePhase(e) {
            return this.combinePhase1(e) + `${r.default.EOL}` + this.combinePhase2(e) + `${r.default.EOL}` + this.combinePhase3(e);
        }
        combinePhase1(e) {
            return `${e.getCode()} ${e.getDescription()}`;
        }
        combinePhase2(e) {
            const t = " At ", r = t + e.getPosition();
            return `Error Message: ${e.getCause()}${r === t ? "" : r}`.includes(i.SPLIT_TAG) ? this.composeCauseAndPosition(e.getCause(), e.getPosition()) : `Error Message: ${e.getCause()}${r === t ? "" : r}`;
        }
        composeCauseAndPosition(e, t) {
            let r = "Error Message: ";
            const n = " At ", o = e.split(i.SPLIT_TAG), a = t.split(i.SPLIT_TAG);
            return r = o.length === a.length ? this.composeCauseAndPositionWithSameLength(o, a, n, r) : this.composeCauseAndPositionWithUnSameLength(o, a, ` ${n}`, r), 
            r;
        }
        composeCauseAndPositionWithSameLength(e, t, n, o) {
            for (let i = 0; i < e.length; i++) {
                o += e[i] + (n + t[i]) + `${r.default.EOL}`;
            }
            return o.slice(0, -`${r.default.EOL}`.length);
        }
        composeCauseAndPositionWithUnSameLength(e, t, n, o) {
            for (let t = 0; t < e.length; t++) {
                o += e[t] + `${r.default.EOL}`;
            }
            for (let e = 0; e < t.length; e++) {
                o += n + t[e] + `${r.default.EOL}`;
            }
            return o.slice(0, -`${r.default.EOL}`.length);
        }
        combinePhase3(e) {
            let t = `${r.default.EOL}* Try the following:${r.default.EOL}`;
            const o = e.getSolutions();
            if (!(o instanceof Array && o.length > 0)) {
                return "";
            }
            o.forEach(e => {
                t += `  > ${e}${r.default.EOL}`;
            });
            const i = e.getMoreInfo();
            if (i) {
                const e = i[(0, n.getOsLanguage)()];
                t += `  > More info: ${e}${r.default.EOL}`;
            }
            return t;
        }
        static combineTrySuggestion(e, t) {
            const n = [ `${r.default.EOL}${r.default.EOL}${s.red}${s.TRY_SUGGESTION_FIRST_LINE}${r.default.EOL}` ];
            return e || n.push(`${s.TRY_SUGGESTION_STACKTRACE_LINE}${r.default.EOL}`), t || n.push(`${s.TRY_SUGGESTION_DEBUG_LINE}${r.default.EOL}`), 
            n.length > 1 ? n.join("") : "";
        }
    }
    e.HvigorErrorCommonAdapter = s, s.red = "[31m", s.green = "[32m", s.yellow = "[33m", 
    s.blue = "[34m", s.reset = "[39m", s.TRY_SUGGESTION_FIRST_LINE = "* Try:", s.TRY_SUGGESTION_STACKTRACE_LINE = "> Run with --stacktrace option to get the stack trace.", 
    s.TRY_SUGGESTION_DEBUG_LINE = "> Run with --debug option to get more log output.";
}(Pc);

var kc = {};

!function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.HvigorErrorInfo = void 0;
    const t = Cc;
    e.HvigorErrorInfo = class {
        constructor(e) {
            this.code = t.UNDEFINED_CODE, this.description = t.UNDEFINED_DESC, this.cause = t.UNDEFINED_CAUSE, 
            this.position = t.UNDEFINED_POS, this.solutions = [], e && (this.code = e.code, 
            this.description = e.description, this.cause = e.cause, this.position = e.position, 
            this.solutions = e.solutions, this.moreInfo = e.moreInfo);
        }
        getCode() {
            return this.code;
        }
        getDescription() {
            return this.description;
        }
        getCause() {
            return this.cause;
        }
        getPosition() {
            return this.position;
        }
        getSolutions() {
            return this.solutions;
        }
        getMoreInfo() {
            return this.moreInfo;
        }
        checkInfo() {
            return this.checkCode() && this.checkCause();
        }
        checkCode() {
            return !!this.code && (8 === this.code.length && !!/[0-9]{8}/g.test(this.code));
        }
        checkCause() {
            return !!this.cause && (this.code === t.UNDEFINED_CODE ? this.cause === t.UNDEFINED_CAUSE : "" !== this.cause && void 0 !== this.cause && null !== this.cause);
        }
    };
}(kc), function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.ErrorUtil = void 0;
    const t = Cc, r = Mc, n = Pc, o = kc;
    class i {
        static getRealError(e) {
            return e instanceof o.HvigorErrorInfo ? e : new o.HvigorErrorInfo({
                code: (null == e ? void 0 : e.code) || t.UNDEFINED_CODE,
                cause: (null == e ? void 0 : e.cause) || t.UNDEFINED_CAUSE,
                description: (null == e ? void 0 : e.description) || t.UNDEFINED_DESC,
                position: (null == e ? void 0 : e.position) || t.UNDEFINED_POS,
                solutions: (null == e ? void 0 : e.solutions) || [],
                moreInfo: null == e ? void 0 : e.moreInfo
            });
        }
        static combinePhase(e) {
            if (!e.description) {
                const r = this.getErrorTypeCodeFromErrorCode(e.code);
                if (r) {
                    const n = t.ErrorCodeDescription[r];
                    e.description = null != n ? n : e.description;
                }
            }
            const r = i.getRealError(e);
            return (new n.HvigorErrorCommonAdapter).convertErrorToLogInfo(r).getMessage();
        }
        static getErrorTypeCodeFromErrorCode(e) {
            if (e && !(e.length < 5)) {
                return e.slice(3, 5);
            }
        }
        static getFirstErrorAdaptorMessage(e) {
            var t;
            return null !== (t = e[0]) && void 0 !== t ? t : {
                message: ""
            };
        }
        static isAdaptorError(e) {
            return e.name === r.AdaptorError.NAME || e instanceof r.AdaptorError;
        }
    }
    e.ErrorUtil = i;
}(Fc);

var Rc = {}, Tc = {};

!function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.ErrorAdaptor = void 0;
    e.ErrorAdaptor = class {};
}(Tc);

var jc = {};

!function(e) {
    var t = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.EIGHT_DIGIT_ERROR_REGEXP = e.MATCH_FIELD_TYPES = e.ERROR_MESSAGE_HIDDEN = e.ERROR_NOT_MATCH = e.DEFAULT_ERROR_CODE = e.ErrorCodeToFile = e.ERROR_INFO_DIRCTORY_PATH = void 0;
    const n = t(r), o = Ac;
    e.ERROR_INFO_DIRCTORY_PATH = n.default.resolve(__dirname, "../../res/error"), e.ErrorCodeToFile = {
        [o.ToolErrorCode.TEC_00]: "global.json",
        [o.ToolErrorCode.TEC_10]: "hvigor.json",
        [o.ToolErrorCode.TEC_11]: "hvigor-ohos-plugin.json",
        [o.ToolErrorCode.TEC_12]: "hvigor-compiler.json",
        [o.ToolErrorCode.TEC_21]: "ark-compiler-toolchain.json",
        [o.ToolErrorCode.TEC_22]: "pack-tool.json",
        [o.ToolErrorCode.TEC_23]: "restool.json",
        [o.ToolErrorCode.TEC_24]: "sign-tool.json"
    }, e.DEFAULT_ERROR_CODE = "00000000", e.ERROR_NOT_MATCH = "The error information does not match.", 
    e.ERROR_MESSAGE_HIDDEN = "Error message is hidden for security reasons", e.MATCH_FIELD_TYPES = [ "none", "id", "checkMessage", "code" ], 
    e.EIGHT_DIGIT_ERROR_REGEXP = {
        PATTERN: "ERROR.{0,12}(?<code>\\d{8})(?!\\d)",
        FLAGS: "gis"
    };
}(jc);

var Lc, Nc, Bc = {}, Uc = {};

function zc() {
    return Lc || (Lc = 1, function(e) {
        Object.defineProperty(e, "__esModule", {
            value: !0
        }), e.HvigorErrorAdaptor = void 0;
        const t = Tc, r = jc, n = Bc, o = Ac, i = Hc();
        e.HvigorErrorAdaptor = class extends t.ErrorAdaptor {
            constructor(e, t = "id") {
                super(), this._hvigorError = new n.HvigorError(this.getErrorCodeJsonPath(), {
                    field: t,
                    value: e
                });
            }
            getErrorMessage() {
                var e;
                return [ {
                    timestamp: this._hvigorError.timestamp,
                    id: this._hvigorError.id,
                    code: this._hvigorError.code,
                    originMessage: this._hvigorError.originMessage,
                    originSolutions: this._hvigorError.originSolutions,
                    moreInfo: this._hvigorError.moreInfo,
                    stack: this._hvigorError.stack,
                    message: null !== (e = this._hvigorError.message) && void 0 !== e ? e : r.ERROR_NOT_MATCH,
                    solutions: this._hvigorError.solutions,
                    checkMessage: this._hvigorError.checkMessage
                } ];
            }
            getErrorCodeJsonPath() {
                return [ (0, i.getErrorInfoFilePath)(this.getToolErrorCode()) ];
            }
            getToolErrorCode() {
                return o.ToolErrorCode.TEC_10;
            }
            formatMessage(...e) {
                return this._hvigorError.formatMessage(...e), this;
            }
            formatSolutions(e, ...t) {
                return this._hvigorError.formatSolutions(e, ...t), this;
            }
            isMatchSuccess() {
                return this._hvigorError.isMatchSuccess();
            }
        };
    }(Rc)), Rc;
}

function Hc() {
    return Nc || (Nc = 1, function(e) {
        var t = g && g.__importDefault || function(e) {
            return e && e.__esModule ? e : {
                default: e
            };
        };
        Object.defineProperty(e, "__esModule", {
            value: !0
        }), e.errorCode2Description = e.formatErrorAdaptorExport = e.getErrorInfoFilePath = void 0;
        const n = t(r), o = Cc, i = Fc, a = zc(), s = jc, u = Ac;
        e.getErrorInfoFilePath = function(e) {
            var t;
            const r = null !== (t = s.ErrorCodeToFile[e]) && void 0 !== t ? t : s.ErrorCodeToFile[u.ToolErrorCode.TEC_00];
            return n.default.resolve(s.ERROR_INFO_DIRCTORY_PATH, r);
        }, e.formatErrorAdaptorExport = function(e, t, r) {
            let n = new a.HvigorErrorAdaptor(e);
            return t && (n = n.formatMessage(...t)), r && r.forEach((e, t) => {
                n = n.formatSolutions(t, ...e);
            }), n;
        }, e.errorCode2Description = function(e) {
            const t = i.ErrorUtil.getErrorTypeCodeFromErrorCode(e);
            return t ? o.ErrorCodeDescription[t] : "";
        };
    }(Oc)), Oc;
}

!function(e) {
    var r = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.BaseError = void 0;
    const n = r(t), o = jc;
    class i {
        constructor(e, t) {
            this._timestamp = new Date, this._errorJsonPaths = e, this._matchOptions = t, this._errorInfo = this.findErrorInfo(), 
            this._errorInfo ? (this._id = this._errorInfo.id, this._code = this._errorInfo.code, 
            this._moreInfo = this._errorInfo.moreInfo, this._checkMessage = this._errorInfo.checkMessage, 
            this._solutions = this._errorInfo.solutions) : this._code = o.DEFAULT_ERROR_CODE;
        }
        isMatchSuccess() {
            return !!this._errorInfo;
        }
        findErrorInfo() {
            return this.match(this._matchOptions);
        }
        match(e) {
            if (i.validFields.includes(e.field) && e.value) {
                return "id" === e.field ? this.matchById(e.value) : this.matchByField(e);
            }
        }
        matchByField(e) {
            const t = this.getErrorInfoJson();
            if (t) {
                for (const r of Object.keys(t)) {
                    const n = t[r];
                    if ("checkMessage" === e.field) {
                        if (n.checkMessage && e.value.includes(n.checkMessage)) {
                            return this.putIdIntoErrorInfo(r, t[r]);
                        }
                    } else if (e.value === n[e.field]) {
                        return this.putIdIntoErrorInfo(r, t[r]);
                    }
                }
            }
        }
        matchById(e) {
            const t = this.getErrorInfoJson();
            return this.putIdIntoErrorInfo(e, null == t ? void 0 : t[e]);
        }
        putIdIntoErrorInfo(e, t) {
            if (t) {
                return {
                    ...t,
                    id: e
                };
            }
        }
        getErrorInfoJson() {
            return this._errorJsonPaths.length ? this._errorJsonPaths.reduce((e, t) => ({
                ...e,
                ...this.getJsonObj(t)
            }), {}) : void 0;
        }
        getJsonObj(e, t = "utf-8") {
            if (!n.default.existsSync(e)) {
                return;
            }
            const r = n.default.readFileSync(e, {
                encoding: t
            });
            try {
                return JSON.parse(r);
            } catch (e) {
                return;
            }
        }
        get timestamp() {
            return this._timestamp;
        }
        get errorJsonPaths() {
            return this._errorJsonPaths;
        }
        get id() {
            return this._id;
        }
        set id(e) {
            this._id = e;
        }
        get message() {
            return this._message;
        }
        set message(e) {
            this._message = e;
        }
        get solutions() {
            return this._solutions;
        }
        set solutions(e) {
            this._solutions = e;
        }
        get moreInfo() {
            return this._moreInfo;
        }
        set moreInfo(e) {
            this._moreInfo = e;
        }
        get code() {
            return this._code;
        }
        set code(e) {
            this._code = e;
        }
        get stack() {
            return this._stack;
        }
        set stack(e) {
            this._stack = e;
        }
        get checkMessage() {
            return this._checkMessage;
        }
        set checkMessage(e) {
            this._checkMessage = e;
        }
        get errorInfo() {
            return this._errorInfo;
        }
    }
    e.BaseError = i, i.validFields = [ "id", "checkMessage", "code" ];
}(Uc), function(e) {
    var t = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.HvigorError = void 0;
    const r = t(l), n = Uc;
    e.HvigorError = class extends n.BaseError {
        constructor(e, t) {
            super(e, t), this._errorInfo && (this._originMessage = this._errorInfo.message, 
            this._message = this._errorInfo.message, this._originSolutions = this._errorInfo.solutions);
        }
        formatMessage(...e) {
            this.originMessage && (this._message = r.default.format(this._originMessage, ...e));
        }
        formatSolutions(e, ...t) {
            this._originSolutions && this._originSolutions[e] && this._solutions && (this._solutions[e] = r.default.format(this._originSolutions[e], ...t));
        }
        get originMessage() {
            return this._originMessage;
        }
        get originSolutions() {
            return this._originSolutions;
        }
    };
}(Bc);

var $c = {}, Gc = {};

!function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.ThirdPartyError = void 0;
    const t = Uc;
    e.ThirdPartyError = class extends t.BaseError {
        constructor(e, t) {
            super(e, t), this._errorInfo && (this._toolName = this._errorInfo.toolName), "checkMessage" === t.field && (this._message = t.value);
        }
        get toolName() {
            return this._toolName;
        }
        set toolName(e) {
            this._toolName = e;
        }
    };
}(Gc), function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.ThirdPartyErrorAdaptor = void 0;
    const t = Tc, r = jc, n = Gc, o = Ac, i = Hc();
    class a extends t.ErrorAdaptor {
        constructor(e, t = "checkMessage") {
            if (super(), this._thirdPartyError = new n.ThirdPartyError(this.getErrorCodeJsonPath(), {
                field: t,
                value: e
            }), "checkMessage" === t) {
                this.errorMessage = e, this.analyze(e);
            } else if ("none" === t) {
                const t = e;
                this._thirdPartyError.code = t.code, this._thirdPartyError.message = t.message, 
                this._thirdPartyError.solutions = t.solutions, this._thirdPartyError.toolName = t.toolName;
            }
        }
        analyze(e) {
            if (!e.includes(a.SOLUTIONS)) {
                return;
            }
            const t = e.split(a.SOLUTIONS);
            this._thirdPartyError.message = t[0].trimEnd(), this._thirdPartyError.solutions = t[1].split(">").map(e => e.trimEnd().trim()).filter(e => e.length > 0), 
            this.checkEs2abc();
        }
        checkEs2abc() {
            const e = this._thirdPartyError.solutions;
            if (e && e[(null == e ? void 0 : e.length) - 1].includes("The size of programs is expected to be")) {
                const t = e[(null == e ? void 0 : e.length) - 1], r = t.indexOf("The size of programs is expected");
                e[(null == e ? void 0 : e.length) - 1] = `${t.substring(0, r)}\n\n${t.substring(r)}`, 
                this._thirdPartyError.solutions = e;
            }
        }
        getErrorMessage() {
            var e, t;
            return [ {
                timestamp: this._thirdPartyError.timestamp,
                id: this._thirdPartyError.id,
                code: this._thirdPartyError.code,
                moreInfo: this._thirdPartyError.moreInfo,
                stack: this._thirdPartyError.stack,
                message: null !== (t = null !== (e = this._thirdPartyError.message) && void 0 !== e ? e : this.errorMessage) && void 0 !== t ? t : r.ERROR_NOT_MATCH,
                solutions: this._thirdPartyError.solutions,
                checkMessage: this._thirdPartyError.checkMessage
            } ];
        }
        getErrorCodeJsonPath(e) {
            return e ? [ (0, i.getErrorInfoFilePath)(e) ] : a.otherErrorCode.map(e => (0, i.getErrorInfoFilePath)(e));
        }
    }
    e.ThirdPartyErrorAdaptor = a, a.SOLUTIONS = "Solutions:", a.otherErrorCode = [ o.ToolErrorCode.TEC_23, o.ToolErrorCode.TEC_24 ];
}($c), function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.ArkTsErrorAdaptor = void 0;
    const t = Ac, r = Hc(), n = $c;
    e.ArkTsErrorAdaptor = class extends n.ThirdPartyErrorAdaptor {
        constructor(e, t) {
            super(e, t);
        }
        getErrorCodeJsonPath() {
            return [ (0, r.getErrorInfoFilePath)(t.ToolErrorCode.TEC_21) ];
        }
    };
}(Sc);

var Wc = {};

!function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.HvigorGlobalErrorAdaptor = void 0;
    const t = Ac, r = Hc(), n = zc();
    class o extends n.HvigorErrorAdaptor {
        constructor(e) {
            super(e, "checkMessage"), this._hvigorError.message ? "%s" === this._hvigorError.message && this.formatMessage(e) : this._hvigorError.message = e;
        }
        getErrorCodeJsonPath() {
            return [ (0, r.getErrorInfoFilePath)(t.ToolErrorCode.TEC_00) ];
        }
    }
    e.HvigorGlobalErrorAdaptor = o;
}(Wc);

var Vc = {};

!function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.HvigorOhosPluginAdaptor = void 0;
    const t = Ac, r = Hc(), n = zc();
    class o extends n.HvigorErrorAdaptor {
        getErrorCodeJsonPath() {
            return [ (0, r.getErrorInfoFilePath)(t.ToolErrorCode.TEC_11) ];
        }
    }
    e.HvigorOhosPluginAdaptor = o;
}(Vc);

var Kc = {}, qc = {}, Jc = {};

!function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.EightDigitErrorCodeAdaptor = void 0;
    const t = Tc, r = jc;
    e.EightDigitErrorCodeAdaptor = class extends t.ErrorAdaptor {
        constructor(e) {
            super(), this._errorMsg = e;
        }
        getErrorInfos() {
            var e;
            const t = new RegExp(r.EIGHT_DIGIT_ERROR_REGEXP.PATTERN, r.EIGHT_DIGIT_ERROR_REGEXP.FLAGS), n = this._errorMsg.matchAll(t), o = [];
            for (const t of n) {
                const n = {
                    code: (null === (e = t.groups) || void 0 === e ? void 0 : e.code) || r.DEFAULT_ERROR_CODE
                };
                o.push(n);
            }
            return o;
        }
        getErrorMessage() {
            const e = this.getErrorInfos().map(e => ({
                timestamp: new Date,
                code: e.code,
                message: ""
            }));
            return e[0] && (e[0].message = this._errorMsg), e;
        }
    };
}(Jc);

var Xc = {};

!function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.PackToolAdaptor = void 0;
    const t = Ac, r = Hc(), n = $c;
    e.PackToolAdaptor = class extends n.ThirdPartyErrorAdaptor {
        constructor(e) {
            super(e);
        }
        analyze(e) {
            let t = "";
            e.split("\r\n").forEach(e => {
                if (e.includes(n.ThirdPartyErrorAdaptor.SOLUTIONS)) {
                    const t = e.split(n.ThirdPartyErrorAdaptor.SOLUTIONS);
                    t.length >= 2 && (this._thirdPartyError.solutions = t[1].split(">").map(e => e.trimEnd().trim()).filter(e => e.length > 0));
                } else {
                    t += `${e}\r\n`;
                }
            }), this._thirdPartyError.message = t.trimEnd();
        }
        getErrorCodeJsonPath() {
            return [ (0, r.getErrorInfoFilePath)(t.ToolErrorCode.TEC_22) ];
        }
    };
}(Xc), function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.AdaptorFactory = void 0;
    const t = jc, r = Jc, n = Wc, o = Xc, i = $c;
    class a {
        static createErrorAdaptor(e) {
            return a.hasErrorCode(e) ? new r.EightDigitErrorCodeAdaptor(e) : a.isThirdPartyError(e) ? e.includes("BundleTool") ? new o.PackToolAdaptor(e) : new i.ThirdPartyErrorAdaptor(e) : new n.HvigorGlobalErrorAdaptor(e);
        }
        static isThirdPartyError(e) {
            return e.includes(i.ThirdPartyErrorAdaptor.SOLUTIONS);
        }
        static hasErrorCode(e) {
            return new RegExp(t.EIGHT_DIGIT_ERROR_REGEXP.PATTERN, t.EIGHT_DIGIT_ERROR_REGEXP.FLAGS).test(e);
        }
    }
    e.AdaptorFactory = a;
}(qc), function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.MixAdaptor = void 0;
    const t = qc;
    e.MixAdaptor = class {
        constructor(e) {
            this.adaptor = t.AdaptorFactory.createErrorAdaptor(e);
        }
        getErrorMessage() {
            return this.adaptor.getErrorMessage();
        }
    };
}(Kc);

var Zc, Yc, Qc, ef, tf, rf = {}, nf = {}, of = {}, af = {}, sf = {}, uf = {}, lf = {}, cf = {
    exports: {}
}, ff = {
    exports: {}
};

function df() {
    if (Yc) {
        return Zc;
    }
    Yc = 1;
    var e = 1e3, t = 60 * e, r = 60 * t, n = 24 * r, o = 7 * n, i = 365.25 * n;
    function a(e, t, r, n) {
        var o = t >= 1.5 * r;
        return Math.round(e / r) + " " + n + (o ? "s" : "");
    }
    return Zc = function(s, u) {
        u = u || {};
        var l = typeof s;
        if ("string" === l && s.length > 0) {
            return function(a) {
                if ((a = String(a)).length > 100) {
                    return;
                }
                var s = /^(-?(?:\d+)?\.?\d+) *(milliseconds?|msecs?|ms|seconds?|secs?|s|minutes?|mins?|m|hours?|hrs?|h|days?|d|weeks?|w|years?|yrs?|y)?$/i.exec(a);
                if (!s) {
                    return;
                }
                var u = parseFloat(s[1]);
                switch ((s[2] || "ms").toLowerCase()) {
                  case "years":
                  case "year":
                  case "yrs":
                  case "yr":
                  case "y":
                    return u * i;

                  case "weeks":
                  case "week":
                  case "w":
                    return u * o;

                  case "days":
                  case "day":
                  case "d":
                    return u * n;

                  case "hours":
                  case "hour":
                  case "hrs":
                  case "hr":
                  case "h":
                    return u * r;

                  case "minutes":
                  case "minute":
                  case "mins":
                  case "min":
                  case "m":
                    return u * t;

                  case "seconds":
                  case "second":
                  case "secs":
                  case "sec":
                  case "s":
                    return u * e;

                  case "milliseconds":
                  case "millisecond":
                  case "msecs":
                  case "msec":
                  case "ms":
                    return u;

                  default:
                    return;
                }
            }(s);
        }
        if ("number" === l && isFinite(s)) {
            return u.long ? function(o) {
                var i = Math.abs(o);
                if (i >= n) {
                    return a(o, i, n, "day");
                }
                if (i >= r) {
                    return a(o, i, r, "hour");
                }
                if (i >= t) {
                    return a(o, i, t, "minute");
                }
                if (i >= e) {
                    return a(o, i, e, "second");
                }
                return o + " ms";
            }(s) : function(o) {
                var i = Math.abs(o);
                if (i >= n) {
                    return Math.round(o / n) + "d";
                }
                if (i >= r) {
                    return Math.round(o / r) + "h";
                }
                if (i >= t) {
                    return Math.round(o / t) + "m";
                }
                if (i >= e) {
                    return Math.round(o / e) + "s";
                }
                return o + "ms";
            }(s);
        }
        throw new Error("val is not a non-empty string or a valid number. val=" + JSON.stringify(s));
    }, Zc;
}

function pf() {
    if (ef) {
        return Qc;
    }
    return ef = 1, Qc = function(e) {
        function t(e) {
            let n, o, i, a = null;
            function s(...e) {
                if (!s.enabled) {
                    return;
                }
                const r = s, o = Number(new Date), i = o - (n || o);
                r.diff = i, r.prev = n, r.curr = o, n = o, e[0] = t.coerce(e[0]), "string" != typeof e[0] && e.unshift("%O");
                let a = 0;
                e[0] = e[0].replace(/%([a-zA-Z%])/g, (n, o) => {
                    if ("%%" === n) {
                        return "%";
                    }
                    a++;
                    const i = t.formatters[o];
                    if ("function" == typeof i) {
                        const t = e[a];
                        n = i.call(r, t), e.splice(a, 1), a--;
                    }
                    return n;
                }), t.formatArgs.call(r, e);
                (r.log || t.log).apply(r, e);
            }
            return s.namespace = e, s.useColors = t.useColors(), s.color = t.selectColor(e), 
            s.extend = r, s.destroy = t.destroy, Object.defineProperty(s, "enabled", {
                enumerable: !0,
                configurable: !1,
                get: () => null !== a ? a : (o !== t.namespaces && (o = t.namespaces, i = t.enabled(e)), 
                i),
                set: e => {
                    a = e;
                }
            }), "function" == typeof t.init && t.init(s), s;
        }
        function r(e, r) {
            const n = t(this.namespace + (void 0 === r ? ":" : r) + e);
            return n.log = this.log, n;
        }
        function n(e, t) {
            let r = 0, n = 0, o = -1, i = 0;
            for (;r < e.length; ) {
                if (n < t.length && (t[n] === e[r] || "*" === t[n])) {
                    "*" === t[n] ? (o = n, i = r, n++) : (r++, n++);
                } else {
                    if (-1 === o) {
                        return !1;
                    }
                    n = o + 1, i++, r = i;
                }
            }
            for (;n < t.length && "*" === t[n]; ) {
                n++;
            }
            return n === t.length;
        }
        return t.debug = t, t.default = t, t.coerce = function(e) {
            if (e instanceof Error) {
                return e.stack || e.message;
            }
            return e;
        }, t.disable = function() {
            const e = [ ...t.names, ...t.skips.map(e => "-" + e) ].join(",");
            return t.enable(""), e;
        }, t.enable = function(e) {
            t.save(e), t.namespaces = e, t.names = [], t.skips = [];
            const r = ("string" == typeof e ? e : "").trim().replace(/\s+/g, ",").split(",").filter(Boolean);
            for (const e of r) {
                "-" === e[0] ? t.skips.push(e.slice(1)) : t.names.push(e);
            }
        }, t.enabled = function(e) {
            for (const r of t.skips) {
                if (n(e, r)) {
                    return !1;
                }
            }
            for (const r of t.names) {
                if (n(e, r)) {
                    return !0;
                }
            }
            return !1;
        }, t.humanize = df(), t.destroy = function() {
            console.warn("Instance method `debug.destroy()` is deprecated and no longer does anything. It will be removed in the next major version of `debug`.");
        }, Object.keys(e).forEach(r => {
            t[r] = e[r];
        }), t.names = [], t.skips = [], t.formatters = {}, t.selectColor = function(e) {
            let r = 0;
            for (let t = 0; t < e.length; t++) {
                r = (r << 5) - r + e.charCodeAt(t), r |= 0;
            }
            return t.colors[Math.abs(r) % t.colors.length];
        }, t.enable(t.load()), t;
    }, Qc;
}

var hf, vf, gf, mf, yf, _f = {
    exports: {}
};

function Ef() {
    return vf ? hf : (vf = 1, hf = (e, t = process.argv) => {
        const r = e.startsWith("-") ? "" : 1 === e.length ? "-" : "--", n = t.indexOf(r + e), o = t.indexOf("--");
        return -1 !== n && (-1 === o || n < o);
    });
}

"undefined" == typeof process || "renderer" === process.type || !0 === process.browser || process.__nwjs ? cf.exports = (tf || (tf = 1, 
function(e, t) {
    t.formatArgs = function(t) {
        if (t[0] = (this.useColors ? "%c" : "") + this.namespace + (this.useColors ? " %c" : " ") + t[0] + (this.useColors ? "%c " : " ") + "+" + e.exports.humanize(this.diff), 
        !this.useColors) {
            return;
        }
        const r = "color: " + this.color;
        t.splice(1, 0, r, "color: inherit");
        let n = 0, o = 0;
        t[0].replace(/%[a-zA-Z%]/g, e => {
            "%%" !== e && (n++, "%c" === e && (o = n));
        }), t.splice(o, 0, r);
    }, t.save = function(e) {
        try {
            e ? t.storage.setItem("debug", e) : t.storage.removeItem("debug");
        } catch (e) {}
    }, t.load = function() {
        let e;
        try {
            e = t.storage.getItem("debug") || t.storage.getItem("DEBUG");
        } catch (e) {}
        return !e && "undefined" != typeof process && "env" in process && (e = process.env.DEBUG), 
        e;
    }, t.useColors = function() {
        if ("undefined" != typeof window && window.process && ("renderer" === window.process.type || window.process.__nwjs)) {
            return !0;
        }
        if ("undefined" != typeof navigator && navigator.userAgent && navigator.userAgent.toLowerCase().match(/(edge|trident)\/(\d+)/)) {
            return !1;
        }
        let e;
        return "undefined" != typeof document && document.documentElement && document.documentElement.style && document.documentElement.style.WebkitAppearance || "undefined" != typeof window && window.console && (window.console.firebug || window.console.exception && window.console.table) || "undefined" != typeof navigator && navigator.userAgent && (e = navigator.userAgent.toLowerCase().match(/firefox\/(\d+)/)) && parseInt(e[1], 10) >= 31 || "undefined" != typeof navigator && navigator.userAgent && navigator.userAgent.toLowerCase().match(/applewebkit\/(\d+)/);
    }, t.storage = function() {
        try {
            return localStorage;
        } catch (e) {}
    }(), t.destroy = (() => {
        let e = !1;
        return () => {
            e || (e = !0, console.warn("Instance method `debug.destroy()` is deprecated and no longer does anything. It will be removed in the next major version of `debug`."));
        };
    })(), t.colors = [ "#0000CC", "#0000FF", "#0033CC", "#0033FF", "#0066CC", "#0066FF", "#0099CC", "#0099FF", "#00CC00", "#00CC33", "#00CC66", "#00CC99", "#00CCCC", "#00CCFF", "#3300CC", "#3300FF", "#3333CC", "#3333FF", "#3366CC", "#3366FF", "#3399CC", "#3399FF", "#33CC00", "#33CC33", "#33CC66", "#33CC99", "#33CCCC", "#33CCFF", "#6600CC", "#6600FF", "#6633CC", "#6633FF", "#66CC00", "#66CC33", "#9900CC", "#9900FF", "#9933CC", "#9933FF", "#99CC00", "#99CC33", "#CC0000", "#CC0033", "#CC0066", "#CC0099", "#CC00CC", "#CC00FF", "#CC3300", "#CC3333", "#CC3366", "#CC3399", "#CC33CC", "#CC33FF", "#CC6600", "#CC6633", "#CC9900", "#CC9933", "#CCCC00", "#CCCC33", "#FF0000", "#FF0033", "#FF0066", "#FF0099", "#FF00CC", "#FF00FF", "#FF3300", "#FF3333", "#FF3366", "#FF3399", "#FF33CC", "#FF33FF", "#FF6600", "#FF6633", "#FF9900", "#FF9933", "#FFCC00", "#FFCC33" ], 
    t.log = console.debug || console.log || (() => {}), e.exports = pf()(t);
    const {formatters: r} = e.exports;
    r.j = function(e) {
        try {
            return JSON.stringify(e);
        } catch (e) {
            return "[UnexpectedJSONParseError]: " + e.message;
        }
    };
}(ff, ff.exports)), ff.exports) : cf.exports = (yf || (yf = 1, function(e, t) {
    const r = f, n = l;
    t.init = function(e) {
        e.inspectOpts = {};
        const r = Object.keys(t.inspectOpts);
        for (let n = 0; n < r.length; n++) {
            e.inspectOpts[r[n]] = t.inspectOpts[r[n]];
        }
    }, t.log = function(...e) {
        return process.stderr.write(n.formatWithOptions(t.inspectOpts, ...e) + "\n");
    }, t.formatArgs = function(r) {
        const {namespace: n, useColors: o} = this;
        if (o) {
            const t = this.color, o = "[3" + (t < 8 ? t : "8;5;" + t), i = `  ${o};1m${n} [0m`;
            r[0] = i + r[0].split("\n").join("\n" + i), r.push(o + "m+" + e.exports.humanize(this.diff) + "[0m");
        } else {
            r[0] = (t.inspectOpts.hideDate ? "" : (new Date).toISOString() + " ") + n + " " + r[0];
        }
    }, t.save = function(e) {
        e ? process.env.DEBUG = e : delete process.env.DEBUG;
    }, t.load = function() {
        return process.env.DEBUG;
    }, t.useColors = function() {
        return "colors" in t.inspectOpts ? Boolean(t.inspectOpts.colors) : r.isatty(process.stderr.fd);
    }, t.destroy = n.deprecate(() => {}, "Instance method `debug.destroy()` is deprecated and no longer does anything. It will be removed in the next major version of `debug`."), 
    t.colors = [ 6, 2, 3, 4, 5, 1 ];
    try {
        const e = function() {
            if (mf) {
                return gf;
            }
            mf = 1;
            const e = a, t = f, r = Ef(), {env: n} = process;
            let o;
            function i(e) {
                return 0 !== e && {
                    level: e,
                    hasBasic: !0,
                    has256: e >= 2,
                    has16m: e >= 3
                };
            }
            function s(t, i) {
                if (0 === o) {
                    return 0;
                }
                if (r("color=16m") || r("color=full") || r("color=truecolor")) {
                    return 3;
                }
                if (r("color=256")) {
                    return 2;
                }
                if (t && !i && void 0 === o) {
                    return 0;
                }
                const a = o || 0;
                if ("dumb" === n.TERM) {
                    return a;
                }
                if ("win32" === process.platform) {
                    const t = e.release().split(".");
                    return Number(t[0]) >= 10 && Number(t[2]) >= 10586 ? Number(t[2]) >= 14931 ? 3 : 2 : 1;
                }
                if ("CI" in n) {
                    return [ "TRAVIS", "CIRCLECI", "APPVEYOR", "GITLAB_CI", "GITHUB_ACTIONS", "BUILDKITE" ].some(e => e in n) || "codeship" === n.CI_NAME ? 1 : a;
                }
                if ("TEAMCITY_VERSION" in n) {
                    return /^(9\.(0*[1-9]\d*)\.|\d{2,}\.)/.test(n.TEAMCITY_VERSION) ? 1 : 0;
                }
                if ("truecolor" === n.COLORTERM) {
                    return 3;
                }
                if ("TERM_PROGRAM" in n) {
                    const e = parseInt((n.TERM_PROGRAM_VERSION || "").split(".")[0], 10);
                    switch (n.TERM_PROGRAM) {
                      case "iTerm.app":
                        return e >= 3 ? 3 : 2;

                      case "Apple_Terminal":
                        return 2;
                    }
                }
                return /-256(color)?$/i.test(n.TERM) ? 2 : /^screen|^xterm|^vt100|^vt220|^rxvt|color|ansi|cygwin|linux/i.test(n.TERM) || "COLORTERM" in n ? 1 : a;
            }
            return r("no-color") || r("no-colors") || r("color=false") || r("color=never") ? o = 0 : (r("color") || r("colors") || r("color=true") || r("color=always")) && (o = 1), 
            "FORCE_COLOR" in n && (o = "true" === n.FORCE_COLOR ? 1 : "false" === n.FORCE_COLOR ? 0 : 0 === n.FORCE_COLOR.length ? 1 : Math.min(parseInt(n.FORCE_COLOR, 10), 3)), 
            gf = {
                supportsColor: function(e) {
                    return i(s(e, e && e.isTTY));
                },
                stdout: i(s(!0, t.isatty(1))),
                stderr: i(s(!0, t.isatty(2)))
            };
        }();
        e && (e.stderr || e).level >= 2 && (t.colors = [ 20, 21, 26, 27, 32, 33, 38, 39, 40, 41, 42, 43, 44, 45, 56, 57, 62, 63, 68, 69, 74, 75, 76, 77, 78, 79, 80, 81, 92, 93, 98, 99, 112, 113, 128, 129, 134, 135, 148, 149, 160, 161, 162, 163, 164, 165, 166, 167, 168, 169, 170, 171, 172, 173, 178, 179, 184, 185, 196, 197, 198, 199, 200, 201, 202, 203, 204, 205, 206, 207, 208, 209, 214, 215, 220, 221 ]);
    } catch (e) {}
    t.inspectOpts = Object.keys(process.env).filter(e => /^debug_/i.test(e)).reduce((e, t) => {
        const r = t.substring(6).toLowerCase().replace(/_([a-z])/g, (e, t) => t.toUpperCase());
        let n = process.env[t];
        return n = !!/^(yes|on|true|enabled)$/i.test(n) || !/^(no|off|false|disabled)$/i.test(n) && ("null" === n ? null : Number(n)), 
        e[r] = n, e;
    }, {}), e.exports = pf()(t);
    const {formatters: o} = e.exports;
    o.o = function(e) {
        return this.inspectOpts.colors = this.useColors, n.inspect(e, this.inspectOpts).split("\n").map(e => e.trim()).join(" ");
    }, o.O = function(e) {
        return this.inspectOpts.colors = this.useColors, n.inspect(e, this.inspectOpts);
    };
}(_f, _f.exports)), _f.exports);

var bf = cf.exports, wf = function(e) {
    return (e = e || {}).circles ? function(e) {
        var t = [], r = [];
        return e.proto ? function e(o) {
            if ("object" != typeof o || null === o) {
                return o;
            }
            if (o instanceof Date) {
                return new Date(o);
            }
            if (Array.isArray(o)) {
                return n(o, e);
            }
            if (o instanceof Map) {
                return new Map(n(Array.from(o), e));
            }
            if (o instanceof Set) {
                return new Set(n(Array.from(o), e));
            }
            var i = {};
            for (var a in t.push(o), r.push(i), o) {
                var s = o[a];
                if ("object" != typeof s || null === s) {
                    i[a] = s;
                } else if (s instanceof Date) {
                    i[a] = new Date(s);
                } else if (s instanceof Map) {
                    i[a] = new Map(n(Array.from(s), e));
                } else if (s instanceof Set) {
                    i[a] = new Set(n(Array.from(s), e));
                } else if (ArrayBuffer.isView(s)) {
                    i[a] = Df(s);
                } else {
                    var u = t.indexOf(s);
                    i[a] = -1 !== u ? r[u] : e(s);
                }
            }
            return t.pop(), r.pop(), i;
        } : function e(o) {
            if ("object" != typeof o || null === o) {
                return o;
            }
            if (o instanceof Date) {
                return new Date(o);
            }
            if (Array.isArray(o)) {
                return n(o, e);
            }
            if (o instanceof Map) {
                return new Map(n(Array.from(o), e));
            }
            if (o instanceof Set) {
                return new Set(n(Array.from(o), e));
            }
            var i = {};
            for (var a in t.push(o), r.push(i), o) {
                if (!1 !== Object.hasOwnProperty.call(o, a)) {
                    var s = o[a];
                    if ("object" != typeof s || null === s) {
                        i[a] = s;
                    } else if (s instanceof Date) {
                        i[a] = new Date(s);
                    } else if (s instanceof Map) {
                        i[a] = new Map(n(Array.from(s), e));
                    } else if (s instanceof Set) {
                        i[a] = new Set(n(Array.from(s), e));
                    } else if (ArrayBuffer.isView(s)) {
                        i[a] = Df(s);
                    } else {
                        var u = t.indexOf(s);
                        i[a] = -1 !== u ? r[u] : e(s);
                    }
                }
            }
            return t.pop(), r.pop(), i;
        };
        function n(e, n) {
            for (var o = Object.keys(e), i = new Array(o.length), a = 0; a < o.length; a++) {
                var s = o[a], u = e[s];
                if ("object" != typeof u || null === u) {
                    i[s] = u;
                } else if (u instanceof Date) {
                    i[s] = new Date(u);
                } else if (ArrayBuffer.isView(u)) {
                    i[s] = Df(u);
                } else {
                    var l = t.indexOf(u);
                    i[s] = -1 !== l ? r[l] : n(u);
                }
            }
            return i;
        }
    }(e) : e.proto ? function e(r) {
        if ("object" != typeof r || null === r) {
            return r;
        }
        if (r instanceof Date) {
            return new Date(r);
        }
        if (Array.isArray(r)) {
            return t(r, e);
        }
        if (r instanceof Map) {
            return new Map(t(Array.from(r), e));
        }
        if (r instanceof Set) {
            return new Set(t(Array.from(r), e));
        }
        var n = {};
        for (var o in r) {
            var i = r[o];
            "object" != typeof i || null === i ? n[o] = i : i instanceof Date ? n[o] = new Date(i) : i instanceof Map ? n[o] = new Map(t(Array.from(i), e)) : i instanceof Set ? n[o] = new Set(t(Array.from(i), e)) : ArrayBuffer.isView(i) ? n[o] = Df(i) : n[o] = e(i);
        }
        return n;
    } : r;
    function t(e, t) {
        for (var r = Object.keys(e), n = new Array(r.length), o = 0; o < r.length; o++) {
            var i = r[o], a = e[i];
            "object" != typeof a || null === a ? n[i] = a : a instanceof Date ? n[i] = new Date(a) : ArrayBuffer.isView(a) ? n[i] = Df(a) : n[i] = t(a);
        }
        return n;
    }
    function r(e) {
        if ("object" != typeof e || null === e) {
            return e;
        }
        if (e instanceof Date) {
            return new Date(e);
        }
        if (Array.isArray(e)) {
            return t(e, r);
        }
        if (e instanceof Map) {
            return new Map(t(Array.from(e), r));
        }
        if (e instanceof Set) {
            return new Set(t(Array.from(e), r));
        }
        var n = {};
        for (var o in e) {
            if (!1 !== Object.hasOwnProperty.call(e, o)) {
                var i = e[o];
                "object" != typeof i || null === i ? n[o] = i : i instanceof Date ? n[o] = new Date(i) : i instanceof Map ? n[o] = new Map(t(Array.from(i), r)) : i instanceof Set ? n[o] = new Set(t(Array.from(i), r)) : ArrayBuffer.isView(i) ? n[o] = Df(i) : n[o] = r(i);
            }
        }
        return n;
    }
};

function Df(e) {
    return e instanceof Buffer ? Buffer.from(e) : new e.constructor(e.buffer.slice(), e.byteOffset, e.length);
}

const Sf = l, Af = bf("log4js:configuration"), Of = [], Cf = [], xf = e => !e, Ff = e => e && "object" == typeof e && !Array.isArray(e), Mf = (e, t, r) => {
    (Array.isArray(t) ? t : [ t ]).forEach(t => {
        if (t) {
            throw new Error(`Problem with log4js configuration: (${Sf.inspect(e, {
                depth: 5
            })}) - ${r}`);
        }
    });
};

var Pf = {
    configure: e => {
        Af("New configuration to be validated: ", e), Mf(e, xf(Ff(e)), "must be an object."), 
        Af(`Calling pre-processing listeners (${Of.length})`), Of.forEach(t => t(e)), Af("Configuration pre-processing finished."), 
        Af(`Calling configuration listeners (${Cf.length})`), Cf.forEach(t => t(e)), Af("Configuration finished.");
    },
    addListener: e => {
        Cf.push(e), Af(`Added listener, now ${Cf.length} listeners`);
    },
    addPreProcessingListener: e => {
        Of.push(e), Af(`Added pre-processing listener, now ${Of.length} listeners`);
    },
    throwExceptionIf: Mf,
    anObject: Ff,
    anInteger: e => e && "number" == typeof e && Number.isInteger(e),
    validIdentifier: e => /^[A-Za-z][A-Za-z0-9_]*$/g.test(e),
    not: xf
}, If = {
    exports: {}
};

!function(e) {
    function t(e, t) {
        for (var r = e.toString(); r.length < t; ) {
            r = "0" + r;
        }
        return r;
    }
    function r(e) {
        return t(e, 2);
    }
    function n(n, o) {
        "string" != typeof n && (o = n, n = e.exports.ISO8601_FORMAT), o || (o = e.exports.now());
        var i = r(o.getDate()), a = r(o.getMonth() + 1), s = r(o.getFullYear()), u = r(s.substring(2, 4)), l = n.indexOf("yyyy") > -1 ? s : u, c = r(o.getHours()), f = r(o.getMinutes()), d = r(o.getSeconds()), p = t(o.getMilliseconds(), 3), h = function(e) {
            var t = Math.abs(e), r = String(Math.floor(t / 60)), n = String(t % 60);
            return r = ("0" + r).slice(-2), n = ("0" + n).slice(-2), 0 === e ? "Z" : (e < 0 ? "+" : "-") + r + ":" + n;
        }(o.getTimezoneOffset());
        return n.replace(/dd/g, i).replace(/MM/g, a).replace(/y{1,4}/g, l).replace(/hh/g, c).replace(/mm/g, f).replace(/ss/g, d).replace(/SSS/g, p).replace(/O/g, h);
    }
    function o(e, t, r, n) {
        e["set" + (n ? "" : "UTC") + t](r);
    }
    e.exports = n, e.exports.asString = n, e.exports.parse = function(t, r, n) {
        if (!t) {
            throw new Error("pattern must be supplied");
        }
        return function(t, r, n) {
            var i = t.indexOf("O") < 0, a = !1, s = [ {
                pattern: /y{1,4}/,
                regexp: "\\d{1,4}",
                fn: function(e, t) {
                    o(e, "FullYear", t, i);
                }
            }, {
                pattern: /MM/,
                regexp: "\\d{1,2}",
                fn: function(e, t) {
                    o(e, "Month", t - 1, i), e.getMonth() !== t - 1 && (a = !0);
                }
            }, {
                pattern: /dd/,
                regexp: "\\d{1,2}",
                fn: function(e, t) {
                    a && o(e, "Month", e.getMonth() - 1, i), o(e, "Date", t, i);
                }
            }, {
                pattern: /hh/,
                regexp: "\\d{1,2}",
                fn: function(e, t) {
                    o(e, "Hours", t, i);
                }
            }, {
                pattern: /mm/,
                regexp: "\\d\\d",
                fn: function(e, t) {
                    o(e, "Minutes", t, i);
                }
            }, {
                pattern: /ss/,
                regexp: "\\d\\d",
                fn: function(e, t) {
                    o(e, "Seconds", t, i);
                }
            }, {
                pattern: /SSS/,
                regexp: "\\d\\d\\d",
                fn: function(e, t) {
                    o(e, "Milliseconds", t, i);
                }
            }, {
                pattern: /O/,
                regexp: "[+-]\\d{1,2}:?\\d{2}?|Z",
                fn: function(e, t) {
                    t = "Z" === t ? 0 : t.replace(":", "");
                    var r = Math.abs(t), n = (t > 0 ? -1 : 1) * (r % 100 + 60 * Math.floor(r / 100));
                    e.setUTCMinutes(e.getUTCMinutes() + n);
                }
            } ], u = s.reduce(function(e, t) {
                return t.pattern.test(e.regexp) ? (t.index = e.regexp.match(t.pattern).index, e.regexp = e.regexp.replace(t.pattern, "(" + t.regexp + ")")) : t.index = -1, 
                e;
            }, {
                regexp: t,
                index: []
            }), l = s.filter(function(e) {
                return e.index > -1;
            });
            l.sort(function(e, t) {
                return e.index - t.index;
            });
            var c = new RegExp(u.regexp).exec(r);
            if (c) {
                var f = n || e.exports.now();
                return l.forEach(function(e, t) {
                    e.fn(f, c[t + 1]);
                }), f;
            }
            throw new Error("String '" + r + "' could not be parsed as '" + t + "'");
        }(t, r, n);
    }, e.exports.now = function() {
        return new Date;
    }, e.exports.ISO8601_FORMAT = "yyyy-MM-ddThh:mm:ss.SSS", e.exports.ISO8601_WITH_TZ_OFFSET_FORMAT = "yyyy-MM-ddThh:mm:ss.SSSO", 
    e.exports.DATETIME_FORMAT = "dd MM yyyy hh:mm:ss.SSS", e.exports.ABSOLUTETIME_FORMAT = "hh:mm:ss.SSS";
}(If);

var kf = If.exports;

const Rf = kf, Tf = a, jf = l, Lf = r, Nf = d, Bf = bf("log4js:layouts"), Uf = {
    bold: [ 1, 22 ],
    italic: [ 3, 23 ],
    underline: [ 4, 24 ],
    inverse: [ 7, 27 ],
    white: [ 37, 39 ],
    grey: [ 90, 39 ],
    black: [ 90, 39 ],
    blue: [ 34, 39 ],
    cyan: [ 36, 39 ],
    green: [ 32, 39 ],
    magenta: [ 35, 39 ],
    red: [ 91, 39 ],
    yellow: [ 33, 39 ]
};

function zf(e) {
    return e ? `[${Uf[e][0]}m` : "";
}

function Hf(e) {
    return e ? `[${Uf[e][1]}m` : "";
}

function $f(e, t) {
    return r = jf.format("[%s] [%s] %s - ", Rf.asString(e.startTime), e.level.toString(), e.categoryName), 
    zf(n = t) + r + Hf(n);
    var r, n;
}

function Gf(e) {
    return $f(e) + jf.format(...e.data);
}

function Wf(e) {
    return $f(e, e.level.colour) + jf.format(...e.data);
}

function Vf(e) {
    return jf.format(...e.data);
}

function Kf(e) {
    return e.data[0];
}

function qf(e, t) {
    const r = /%(-?[0-9]+)?(\.?-?[0-9]+)?([[\]cdhmnprzxXyflosCMAF%])(\{([^}]+)\})?|([^%]+)/;
    function n(e) {
        return e && e.pid ? e.pid.toString() : process.pid.toString();
    }
    e = e || "%r %p %c - %m%n";
    const o = {
        c: function(e, t) {
            let r = e.categoryName;
            if (t) {
                const e = parseInt(t, 10), n = r.split(".");
                e < n.length && (r = n.slice(n.length - e).join("."));
            }
            return r;
        },
        d: function(e, t) {
            let r = Rf.ISO8601_FORMAT;
            if (t) {
                switch (r = t, r) {
                  case "ISO8601":
                  case "ISO8601_FORMAT":
                    r = Rf.ISO8601_FORMAT;
                    break;

                  case "ISO8601_WITH_TZ_OFFSET":
                  case "ISO8601_WITH_TZ_OFFSET_FORMAT":
                    r = Rf.ISO8601_WITH_TZ_OFFSET_FORMAT;
                    break;

                  case "ABSOLUTE":
                    process.emitWarning("Pattern %d{ABSOLUTE} is deprecated in favor of %d{ABSOLUTETIME}. Please use %d{ABSOLUTETIME} instead.", "DeprecationWarning", "log4js-node-DEP0003"), 
                    Bf("[log4js-node-DEP0003]", "DEPRECATION: Pattern %d{ABSOLUTE} is deprecated and replaced by %d{ABSOLUTETIME}.");

                  case "ABSOLUTETIME":
                  case "ABSOLUTETIME_FORMAT":
                    r = Rf.ABSOLUTETIME_FORMAT;
                    break;

                  case "DATE":
                    process.emitWarning("Pattern %d{DATE} is deprecated due to the confusion it causes when used. Please use %d{DATETIME} instead.", "DeprecationWarning", "log4js-node-DEP0004"), 
                    Bf("[log4js-node-DEP0004]", "DEPRECATION: Pattern %d{DATE} is deprecated and replaced by %d{DATETIME}.");

                  case "DATETIME":
                  case "DATETIME_FORMAT":
                    r = Rf.DATETIME_FORMAT;
                }
            }
            return Rf.asString(r, e.startTime);
        },
        h: function() {
            return Tf.hostname().toString();
        },
        m: function(e, t) {
            let r = e.data;
            if (t) {
                const [e, n] = t.split(",");
                r = r.slice(e, n);
            }
            return jf.format(...r);
        },
        n: function() {
            return Tf.EOL;
        },
        p: function(e) {
            return e.level.toString();
        },
        r: function(e) {
            return Rf.asString("hh:mm:ss", e.startTime);
        },
        "[": function(e) {
            return zf(e.level.colour);
        },
        "]": function(e) {
            return Hf(e.level.colour);
        },
        y: function() {
            return n();
        },
        z: n,
        "%": function() {
            return "%";
        },
        x: function(e, r) {
            return void 0 !== t[r] ? "function" == typeof t[r] ? t[r](e) : t[r] : null;
        },
        X: function(e, t) {
            const r = e.context[t];
            return void 0 !== r ? "function" == typeof r ? r(e) : r : null;
        },
        f: function(e, t) {
            let r = e.fileName || "";
            if (r = function(e) {
                const t = "file://";
                return e.startsWith(t) && ("function" == typeof Nf.fileURLToPath ? e = Nf.fileURLToPath(e) : (e = Lf.normalize(e.replace(new RegExp(`^${t}`), "")), 
                "win32" === process.platform && (e = e.startsWith("\\") ? e.slice(1) : Lf.sep + Lf.sep + e))), 
                e;
            }(r), t) {
                const e = parseInt(t, 10), n = r.split(Lf.sep);
                n.length > e && (r = n.slice(-e).join(Lf.sep));
            }
            return r;
        },
        l: function(e) {
            return e.lineNumber ? `${e.lineNumber}` : "";
        },
        o: function(e) {
            return e.columnNumber ? `${e.columnNumber}` : "";
        },
        s: function(e) {
            return e.callStack || "";
        },
        C: function(e) {
            return e.className || "";
        },
        M: function(e) {
            return e.functionName || "";
        },
        A: function(e) {
            return e.functionAlias || "";
        },
        F: function(e) {
            return e.callerName || "";
        }
    };
    function i(e, t, r) {
        return o[e](t, r);
    }
    function a(e, t, r) {
        let n = e;
        return n = function(e, t) {
            let r;
            return e ? (r = parseInt(e.slice(1), 10), r > 0 ? t.slice(0, r) : t.slice(r)) : t;
        }(t, n), n = function(e, t) {
            let r;
            if (e) {
                if ("-" === e.charAt(0)) {
                    for (r = parseInt(e.slice(1), 10); t.length < r; ) {
                        t += " ";
                    }
                } else {
                    for (r = parseInt(e, 10); t.length < r; ) {
                        t = ` ${t}`;
                    }
                }
            }
            return t;
        }(r, n), n;
    }
    return function(t) {
        let n, o = "", s = e;
        for (;null !== (n = r.exec(s)); ) {
            const e = n[1], r = n[2], u = n[3], l = n[5], c = n[6];
            if (c) {
                o += c.toString();
            } else {
                o += a(i(u, t, l), r, e);
            }
            s = s.slice(n.index + n[0].length);
        }
        return o;
    };
}

const Jf = {
    messagePassThrough: () => Vf,
    basic: () => Gf,
    colored: () => Wf,
    coloured: () => Wf,
    pattern: e => qf(e && e.pattern, e && e.tokens),
    dummy: () => Kf
};

var Xf = {
    basicLayout: Gf,
    messagePassThroughLayout: Vf,
    patternLayout: qf,
    colouredLayout: Wf,
    coloredLayout: Wf,
    dummyLayout: Kf,
    addLayout(e, t) {
        Jf[e] = t;
    },
    layout: (e, t) => Jf[e] && Jf[e](t)
};

const Zf = Pf, Yf = [ "white", "grey", "black", "blue", "cyan", "green", "magenta", "red", "yellow" ];

class Qf {
    constructor(e, t, r) {
        this.level = e, this.levelStr = t, this.colour = r;
    }
    toString() {
        return this.levelStr;
    }
    static getLevel(e, t) {
        return e ? e instanceof Qf ? e : (e instanceof Object && e.levelStr && (e = e.levelStr), 
        Qf[e.toString().toUpperCase()] || t) : t;
    }
    static addLevels(e) {
        if (e) {
            Object.keys(e).forEach(t => {
                const r = t.toUpperCase();
                Qf[r] = new Qf(e[t].value, r, e[t].colour);
                const n = Qf.levels.findIndex(e => e.levelStr === r);
                n > -1 ? Qf.levels[n] = Qf[r] : Qf.levels.push(Qf[r]);
            }), Qf.levels.sort((e, t) => e.level - t.level);
        }
    }
    isLessThanOrEqualTo(e) {
        return "string" == typeof e && (e = Qf.getLevel(e)), this.level <= e.level;
    }
    isGreaterThanOrEqualTo(e) {
        return "string" == typeof e && (e = Qf.getLevel(e)), this.level >= e.level;
    }
    isEqualTo(e) {
        return "string" == typeof e && (e = Qf.getLevel(e)), this.level === e.level;
    }
}

Qf.levels = [], Qf.addLevels({
    ALL: {
        value: Number.MIN_VALUE,
        colour: "grey"
    },
    TRACE: {
        value: 5e3,
        colour: "blue"
    },
    DEBUG: {
        value: 1e4,
        colour: "cyan"
    },
    INFO: {
        value: 2e4,
        colour: "green"
    },
    WARN: {
        value: 3e4,
        colour: "yellow"
    },
    ERROR: {
        value: 4e4,
        colour: "red"
    },
    FATAL: {
        value: 5e4,
        colour: "magenta"
    },
    MARK: {
        value: 9007199254740992,
        colour: "grey"
    },
    OFF: {
        value: Number.MAX_VALUE,
        colour: "grey"
    }
}), Zf.addListener(e => {
    const t = e.levels;
    if (t) {
        Zf.throwExceptionIf(e, Zf.not(Zf.anObject(t)), "levels must be an object");
        Object.keys(t).forEach(r => {
            Zf.throwExceptionIf(e, Zf.not(Zf.validIdentifier(r)), `level name "${r}" is not a valid identifier (must start with a letter, only contain A-Z,a-z,0-9,_)`), 
            Zf.throwExceptionIf(e, Zf.not(Zf.anObject(t[r])), `level "${r}" must be an object`), 
            Zf.throwExceptionIf(e, Zf.not(t[r].value), `level "${r}" must have a 'value' property`), 
            Zf.throwExceptionIf(e, Zf.not(Zf.anInteger(t[r].value)), `level "${r}".value must have an integer value`), 
            Zf.throwExceptionIf(e, Zf.not(t[r].colour), `level "${r}" must have a 'colour' property`), 
            Zf.throwExceptionIf(e, Zf.not(Yf.indexOf(t[r].colour) > -1), `level "${r}".colour must be one of ${Yf.join(", ")}`);
        });
    }
}), Zf.addListener(e => {
    Qf.addLevels(e.levels);
});

var ed = Qf, td = {
    exports: {}
}, rd = {};

/*! (c) 2020 Andrea Giammarchi */
const {parse: nd, stringify: od} = JSON, {keys: id} = Object, ad = String, sd = "string", ud = {}, ld = "object", cd = (e, t) => t, fd = e => e instanceof ad ? ad(e) : e, dd = (e, t) => typeof t === sd ? new ad(t) : t, pd = (e, t, r, n) => {
    const o = [];
    for (let i = id(r), {length: a} = i, s = 0; s < a; s++) {
        const a = i[s], u = r[a];
        if (u instanceof ad) {
            const i = e[u];
            typeof i !== ld || t.has(i) ? r[a] = n.call(r, a, i) : (t.add(i), r[a] = ud, o.push({
                k: a,
                a: [ e, t, i, n ]
            }));
        } else {
            r[a] !== ud && (r[a] = n.call(r, a, u));
        }
    }
    for (let {length: e} = o, t = 0; t < e; t++) {
        const {k: e, a: i} = o[t];
        r[e] = n.call(r, e, pd.apply(null, i));
    }
    return r;
}, hd = (e, t, r) => {
    const n = ad(t.push(r) - 1);
    return e.set(r, n), n;
}, vd = (e, t) => {
    const r = nd(e, dd).map(fd), n = r[0], o = t || cd, i = typeof n === ld && n ? pd(r, new Set, n, o) : n;
    return o.call({
        "": i
    }, "", i);
};

rd.parse = vd;

const gd = (e, t, r) => {
    const n = t && typeof t === ld ? (e, r) => "" === e || -1 < t.indexOf(e) ? r : void 0 : t || cd, o = new Map, i = [], a = [];
    let s = +hd(o, i, n.call({
        "": e
    }, "", e)), u = !s;
    for (;s < i.length; ) {
        u = !0, a[s] = od(i[s++], l, r);
    }
    return "[" + a.join(",") + "]";
    function l(e, t) {
        if (u) {
            return u = !u, t;
        }
        const r = n.call(this, e, t);
        switch (typeof r) {
          case ld:
            if (null === r) {
                return r;
            }

          case sd:
            return o.get(r) || hd(o, i, r);
        }
        return r;
    }
};

rd.stringify = gd;

rd.toJSON = e => nd(gd(e));

rd.fromJSON = e => vd(od(e));

const md = rd, yd = ed;

const _d = new class {
    constructor() {
        const e = {
            __LOG4JS_undefined__: void 0,
            __LOG4JS_NaN__: Number("abc"),
            __LOG4JS_Infinity__: 1 / 0,
            "__LOG4JS_-Infinity__": -1 / 0
        };
        this.deMap = e, this.serMap = {}, Object.keys(this.deMap).forEach(e => {
            const t = this.deMap[e];
            this.serMap[t] = e;
        });
    }
    canSerialise(e) {
        return "string" != typeof e && e in this.serMap;
    }
    serialise(e) {
        return this.canSerialise(e) ? this.serMap[e] : e;
    }
    canDeserialise(e) {
        return e in this.deMap;
    }
    deserialise(e) {
        return this.canDeserialise(e) ? this.deMap[e] : e;
    }
};

var Ed = class e {
    constructor(e, t, r, n, o, i) {
        if (this.startTime = new Date, this.categoryName = e, this.data = r, this.level = t, 
        this.context = Object.assign({}, n), this.pid = process.pid, this.error = i, void 0 !== o) {
            if (!o || "object" != typeof o || Array.isArray(o)) {
                throw new TypeError("Invalid location type passed to LoggingEvent constructor");
            }
            this.constructor._getLocationKeys().forEach(e => {
                void 0 !== o[e] && (this[e] = o[e]);
            });
        }
    }
    static _getLocationKeys() {
        return [ "fileName", "lineNumber", "columnNumber", "callStack", "className", "functionName", "functionAlias", "callerName" ];
    }
    serialise() {
        return md.stringify(this, (e, t) => (t instanceof Error && (t = Object.assign({
            message: t.message,
            stack: t.stack
        }, t)), _d.serialise(t)));
    }
    static deserialise(t) {
        let r;
        try {
            const n = md.parse(t, (e, t) => {
                if (t && t.message && t.stack) {
                    const e = new Error(t);
                    Object.keys(t).forEach(r => {
                        e[r] = t[r];
                    }), t = e;
                }
                return _d.deserialise(t);
            });
            this._getLocationKeys().forEach(e => {
                void 0 !== n[e] && (n.location || (n.location = {}), n.location[e] = n[e]);
            }), r = new e(n.categoryName, yd.getLevel(n.level.levelStr), n.data, n.context, n.location, n.error), 
            r.startTime = new Date(n.startTime), r.pid = n.pid, n.cluster && (r.cluster = n.cluster);
        } catch (n) {
            r = new e("log4js", yd.ERROR, [ "Unable to parse log:", t, "because: ", n ]);
        }
        return r;
    }
};

const bd = bf("log4js:clustering"), wd = Ed, Dd = Pf;

let Sd = !1, Ad = null;

try {
    Ad = require("cluster");
} catch (e) {
    bd("cluster module not present"), Sd = !0;
}

const Od = [];

let Cd = !1, xd = "NODE_APP_INSTANCE";

const Fd = () => Cd && "0" === process.env[xd], Md = () => Sd || Ad && Ad.isMaster || Fd(), Pd = e => {
    Od.forEach(t => t(e));
}, Id = (e, t) => {
    if (bd("cluster message received from worker ", e, ": ", t), e.topic && e.data && (t = e, 
    e = void 0), t && t.topic && "log4js:message" === t.topic) {
        bd("received message: ", t.data);
        const e = wd.deserialise(t.data);
        Pd(e);
    }
};

Sd || Dd.addListener(e => {
    Od.length = 0, ({pm2: Cd, disableClustering: Sd, pm2InstanceVar: xd = "NODE_APP_INSTANCE"} = e), 
    bd(`clustering disabled ? ${Sd}`), bd(`cluster.isMaster ? ${Ad && Ad.isMaster}`), 
    bd(`pm2 enabled ? ${Cd}`), bd(`pm2InstanceVar = ${xd}`), bd(`process.env[${xd}] = ${process.env[xd]}`), 
    Cd && process.removeListener("message", Id), Ad && Ad.removeListener && Ad.removeListener("message", Id), 
    Sd || e.disableClustering ? bd("Not listening for cluster messages, because clustering disabled.") : Fd() ? (bd("listening for PM2 broadcast messages"), 
    process.on("message", Id)) : Ad && Ad.isMaster ? (bd("listening for cluster messages"), 
    Ad.on("message", Id)) : bd("not listening for messages, because we are not a master process");
});

var kd = {
    onlyOnMaster: (e, t) => Md() ? e() : t,
    isMaster: Md,
    send: e => {
        Md() ? Pd(e) : (Cd || (e.cluster = {
            workerId: Ad.worker.id,
            worker: process.pid
        }), process.send({
            topic: "log4js:message",
            data: e.serialise()
        }));
    },
    onMessage: e => {
        Od.push(e);
    }
}, Rd = {};

function Td(e) {
    if ("number" == typeof e && Number.isInteger(e)) {
        return e;
    }
    const t = {
        K: 1024,
        M: 1048576,
        G: 1073741824
    }, r = Object.keys(t), n = e.slice(-1).toLocaleUpperCase(), o = e.slice(0, -1).trim();
    if (r.indexOf(n) < 0 || !Number.isInteger(Number(o))) {
        throw Error(`maxLogSize: "${e}" is invalid`);
    }
    return o * t[n];
}

function jd(e) {
    return function(e, t) {
        const r = Object.assign({}, t);
        return Object.keys(e).forEach(n => {
            r[n] && (r[n] = e[n](t[n]));
        }), r;
    }({
        maxLogSize: Td
    }, e);
}

const Ld = {
    dateFile: jd,
    file: jd,
    fileSync: jd
};

Rd.modifyConfig = e => Ld[e.type] ? Ld[e.type](e) : e;

var Nd = {};

const Bd = console.log.bind(console);

Nd.configure = function(e, t) {
    let r = t.colouredLayout;
    return e.layout && (r = t.layout(e.layout.type, e.layout)), function(e, t) {
        return r => {
            Bd(e(r, t));
        };
    }(r, e.timezoneOffset);
};

var Ud = {};

Ud.configure = function(e, t) {
    let r = t.colouredLayout;
    return e.layout && (r = t.layout(e.layout.type, e.layout)), function(e, t) {
        return r => {
            process.stdout.write(`${e(r, t)}\n`);
        };
    }(r, e.timezoneOffset);
};

var zd = {};

zd.configure = function(e, t) {
    let r = t.colouredLayout;
    return e.layout && (r = t.layout(e.layout.type, e.layout)), function(e, t) {
        return r => {
            process.stderr.write(`${e(r, t)}\n`);
        };
    }(r, e.timezoneOffset);
};

var Hd = {};

Hd.configure = function(e, t, r, n) {
    const o = r(e.appender);
    return function(e, t, r, n) {
        const o = n.getLevel(e), i = n.getLevel(t, n.FATAL);
        return e => {
            const t = e.level;
            o.isLessThanOrEqualTo(t) && i.isGreaterThanOrEqualTo(t) && r(e);
        };
    }(e.level, e.maxLevel, o, n);
};

var $d = {};

const Gd = bf("log4js:categoryFilter");

$d.configure = function(e, t, r) {
    const n = r(e.appender);
    return function(e, t) {
        return "string" == typeof e && (e = [ e ]), r => {
            Gd(`Checking ${r.categoryName} against ${e}`), -1 === e.indexOf(r.categoryName) && (Gd("Not excluded, sending to appender"), 
            t(r));
        };
    }(e.exclude, n);
};

var Wd = {};

const Vd = bf("log4js:noLogFilter");

Wd.configure = function(e, t, r) {
    const n = r(e.appender);
    return function(e, t) {
        return r => {
            Vd(`Checking data: ${r.data} against filters: ${e}`), "string" == typeof e && (e = [ e ]), 
            e = e.filter(e => null != e && "" !== e);
            const n = new RegExp(e.join("|"), "i");
            (0 === e.length || r.data.findIndex(e => n.test(e)) < 0) && (Vd("Not excluded, sending to appender"), 
            t(r));
        };
    }(e.exclude, n);
};

var Kd = {}, qd = {
    exports: {}
}, Jd = {}, Xd = {
    fromCallback: function(e) {
        return Object.defineProperty(function() {
            if ("function" != typeof arguments[arguments.length - 1]) {
                return new Promise((t, r) => {
                    arguments[arguments.length] = (e, n) => {
                        if (e) {
                            return r(e);
                        }
                        t(n);
                    }, arguments.length++, e.apply(this, arguments);
                });
            }
            e.apply(this, arguments);
        }, "name", {
            value: e.name
        });
    },
    fromPromise: function(e) {
        return Object.defineProperty(function() {
            const t = arguments[arguments.length - 1];
            if ("function" != typeof t) {
                return e.apply(this, arguments);
            }
            e.apply(this, arguments).then(e => t(null, e), t);
        }, "name", {
            value: e.name
        });
    }
};

!function(e) {
    const t = Xd.fromCallback, r = li, n = [ "access", "appendFile", "chmod", "chown", "close", "copyFile", "fchmod", "fchown", "fdatasync", "fstat", "fsync", "ftruncate", "futimes", "lchown", "lchmod", "link", "lstat", "mkdir", "mkdtemp", "open", "readFile", "readdir", "readlink", "realpath", "rename", "rmdir", "stat", "symlink", "truncate", "unlink", "utimes", "writeFile" ].filter(e => "function" == typeof r[e]);
    Object.keys(r).forEach(t => {
        "promises" !== t && (e[t] = r[t]);
    }), n.forEach(n => {
        e[n] = t(r[n]);
    }), e.exists = function(e, t) {
        return "function" == typeof t ? r.exists(e, t) : new Promise(t => r.exists(e, t));
    }, e.read = function(e, t, n, o, i, a) {
        return "function" == typeof a ? r.read(e, t, n, o, i, a) : new Promise((a, s) => {
            r.read(e, t, n, o, i, (e, t, r) => {
                if (e) {
                    return s(e);
                }
                a({
                    bytesRead: t,
                    buffer: r
                });
            });
        });
    }, e.write = function(e, t, ...n) {
        return "function" == typeof n[n.length - 1] ? r.write(e, t, ...n) : new Promise((o, i) => {
            r.write(e, t, ...n, (e, t, r) => {
                if (e) {
                    return i(e);
                }
                o({
                    bytesWritten: t,
                    buffer: r
                });
            });
        });
    }, "function" == typeof r.realpath.native && (e.realpath.native = t(r.realpath.native));
}(Jd);

const Zd = r;

function Yd(e) {
    return (e = Zd.normalize(Zd.resolve(e)).split(Zd.sep)).length > 0 ? e[0] : null;
}

const Qd = /[<>:"|?*]/;

var ep = function(e) {
    const t = Yd(e);
    return e = e.replace(t, ""), Qd.test(e);
};

const tp = li, rp = r, np = ep, op = parseInt("0777", 8);

var ip = function e(t, r, n, o) {
    if ("function" == typeof r ? (n = r, r = {}) : r && "object" == typeof r || (r = {
        mode: r
    }), "win32" === process.platform && np(t)) {
        const e = new Error(t + " contains invalid WIN32 path characters.");
        return e.code = "EINVAL", n(e);
    }
    let i = r.mode;
    const a = r.fs || tp;
    void 0 === i && (i = op & ~process.umask()), o || (o = null), n = n || function() {}, 
    t = rp.resolve(t), a.mkdir(t, i, i => {
        if (!i) {
            return n(null, o = o || t);
        }
        if ("ENOENT" === i.code) {
            if (rp.dirname(t) === t) {
                return n(i);
            }
            e(rp.dirname(t), r, (o, i) => {
                o ? n(o, i) : e(t, r, n, i);
            });
        } else {
            a.stat(t, (e, t) => {
                e || !t.isDirectory() ? n(i, o) : n(null, o);
            });
        }
    });
};

const ap = li, sp = r, up = ep, lp = parseInt("0777", 8);

var cp = function e(t, r, n) {
    r && "object" == typeof r || (r = {
        mode: r
    });
    let o = r.mode;
    const i = r.fs || ap;
    if ("win32" === process.platform && up(t)) {
        const e = new Error(t + " contains invalid WIN32 path characters.");
        throw e.code = "EINVAL", e;
    }
    void 0 === o && (o = lp & ~process.umask()), n || (n = null), t = sp.resolve(t);
    try {
        i.mkdirSync(t, o), n = n || t;
    } catch (o) {
        if ("ENOENT" === o.code) {
            if (sp.dirname(t) === t) {
                throw o;
            }
            n = e(sp.dirname(t), r, n), e(t, r, n);
        } else {
            let e;
            try {
                e = i.statSync(t);
            } catch (e) {
                throw o;
            }
            if (!e.isDirectory()) {
                throw o;
            }
        }
    }
    return n;
};

const fp = (0, Xd.fromCallback)(ip);

var dp = {
    mkdirs: fp,
    mkdirsSync: cp,
    mkdirp: fp,
    mkdirpSync: cp,
    ensureDir: fp,
    ensureDirSync: cp
};

const pp = li;

var hp = function(e, t, r, n) {
    pp.open(e, "r+", (e, o) => {
        if (e) {
            return n(e);
        }
        pp.futimes(o, t, r, e => {
            pp.close(o, t => {
                n && n(e || t);
            });
        });
    });
}, vp = function(e, t, r) {
    const n = pp.openSync(e, "r+");
    return pp.futimesSync(n, t, r), pp.closeSync(n);
};

const gp = li, mp = r, yp = process.versions.node.split("."), _p = Number.parseInt(yp[0], 10), Ep = Number.parseInt(yp[1], 10), bp = Number.parseInt(yp[2], 10);

function wp() {
    if (_p > 10) {
        return !0;
    }
    if (10 === _p) {
        if (Ep > 5) {
            return !0;
        }
        if (5 === Ep && bp >= 0) {
            return !0;
        }
    }
    return !1;
}

function Dp(e, t) {
    const r = mp.resolve(e).split(mp.sep).filter(e => e), n = mp.resolve(t).split(mp.sep).filter(e => e);
    return r.reduce((e, t, r) => e && n[r] === t, !0);
}

function Sp(e, t, r) {
    return `Cannot ${r} '${e}' to a subdirectory of itself, '${t}'.`;
}

var Ap, Op, Cp = {
    checkPaths: function(e, t, r, n) {
        !function(e, t, r) {
            wp() ? gp.stat(e, {
                bigint: !0
            }, (e, n) => {
                if (e) {
                    return r(e);
                }
                gp.stat(t, {
                    bigint: !0
                }, (e, t) => e ? "ENOENT" === e.code ? r(null, {
                    srcStat: n,
                    destStat: null
                }) : r(e) : r(null, {
                    srcStat: n,
                    destStat: t
                }));
            }) : gp.stat(e, (e, n) => {
                if (e) {
                    return r(e);
                }
                gp.stat(t, (e, t) => e ? "ENOENT" === e.code ? r(null, {
                    srcStat: n,
                    destStat: null
                }) : r(e) : r(null, {
                    srcStat: n,
                    destStat: t
                }));
            });
        }(e, t, (o, i) => {
            if (o) {
                return n(o);
            }
            const {srcStat: a, destStat: s} = i;
            return s && s.ino && s.dev && s.ino === a.ino && s.dev === a.dev ? n(new Error("Source and destination must not be the same.")) : a.isDirectory() && Dp(e, t) ? n(new Error(Sp(e, t, r))) : n(null, {
                srcStat: a,
                destStat: s
            });
        });
    },
    checkPathsSync: function(e, t, r) {
        const {srcStat: n, destStat: o} = function(e, t) {
            let r, n;
            r = wp() ? gp.statSync(e, {
                bigint: !0
            }) : gp.statSync(e);
            try {
                n = wp() ? gp.statSync(t, {
                    bigint: !0
                }) : gp.statSync(t);
            } catch (e) {
                if ("ENOENT" === e.code) {
                    return {
                        srcStat: r,
                        destStat: null
                    };
                }
                throw e;
            }
            return {
                srcStat: r,
                destStat: n
            };
        }(e, t);
        if (o && o.ino && o.dev && o.ino === n.ino && o.dev === n.dev) {
            throw new Error("Source and destination must not be the same.");
        }
        if (n.isDirectory() && Dp(e, t)) {
            throw new Error(Sp(e, t, r));
        }
        return {
            srcStat: n,
            destStat: o
        };
    },
    checkParentPaths: function e(t, r, n, o, i) {
        const a = mp.resolve(mp.dirname(t)), s = mp.resolve(mp.dirname(n));
        if (s === a || s === mp.parse(s).root) {
            return i();
        }
        wp() ? gp.stat(s, {
            bigint: !0
        }, (a, u) => a ? "ENOENT" === a.code ? i() : i(a) : u.ino && u.dev && u.ino === r.ino && u.dev === r.dev ? i(new Error(Sp(t, n, o))) : e(t, r, s, o, i)) : gp.stat(s, (a, u) => a ? "ENOENT" === a.code ? i() : i(a) : u.ino && u.dev && u.ino === r.ino && u.dev === r.dev ? i(new Error(Sp(t, n, o))) : e(t, r, s, o, i));
    },
    checkParentPathsSync: function e(t, r, n, o) {
        const i = mp.resolve(mp.dirname(t)), a = mp.resolve(mp.dirname(n));
        if (a === i || a === mp.parse(a).root) {
            return;
        }
        let s;
        try {
            s = wp() ? gp.statSync(a, {
                bigint: !0
            }) : gp.statSync(a);
        } catch (e) {
            if ("ENOENT" === e.code) {
                return;
            }
            throw e;
        }
        if (s.ino && s.dev && s.ino === r.ino && s.dev === r.dev) {
            throw new Error(Sp(t, n, o));
        }
        return e(t, r, a, o);
    },
    isSrcSubdir: Dp
};

const xp = li, Fp = r, Mp = dp.mkdirsSync, Pp = vp, Ip = Cp;

function kp(e, t, r, n) {
    if (!n.filter || n.filter(t, r)) {
        return function(e, t, r, n) {
            const o = n.dereference ? xp.statSync : xp.lstatSync, i = o(t);
            if (i.isDirectory()) {
                return function(e, t, r, n, o) {
                    if (!t) {
                        return function(e, t, r, n) {
                            return xp.mkdirSync(r), Tp(t, r, n), xp.chmodSync(r, e.mode);
                        }(e, r, n, o);
                    }
                    if (t && !t.isDirectory()) {
                        throw new Error(`Cannot overwrite non-directory '${n}' with directory '${r}'.`);
                    }
                    return Tp(r, n, o);
                }(i, e, t, r, n);
            }
            if (i.isFile() || i.isCharacterDevice() || i.isBlockDevice()) {
                return function(e, t, r, n, o) {
                    return t ? function(e, t, r, n) {
                        if (n.overwrite) {
                            return xp.unlinkSync(r), Rp(e, t, r, n);
                        }
                        if (n.errorOnExist) {
                            throw new Error(`'${r}' already exists`);
                        }
                    }(e, r, n, o) : Rp(e, r, n, o);
                }(i, e, t, r, n);
            }
            if (i.isSymbolicLink()) {
                return function(e, t, r, n) {
                    let o = xp.readlinkSync(t);
                    n.dereference && (o = Fp.resolve(process.cwd(), o));
                    if (e) {
                        let e;
                        try {
                            e = xp.readlinkSync(r);
                        } catch (e) {
                            if ("EINVAL" === e.code || "UNKNOWN" === e.code) {
                                return xp.symlinkSync(o, r);
                            }
                            throw e;
                        }
                        if (n.dereference && (e = Fp.resolve(process.cwd(), e)), Ip.isSrcSubdir(o, e)) {
                            throw new Error(`Cannot copy '${o}' to a subdirectory of itself, '${e}'.`);
                        }
                        if (xp.statSync(r).isDirectory() && Ip.isSrcSubdir(e, o)) {
                            throw new Error(`Cannot overwrite '${e}' with '${o}'.`);
                        }
                        return function(e, t) {
                            return xp.unlinkSync(t), xp.symlinkSync(e, t);
                        }(o, r);
                    }
                    return xp.symlinkSync(o, r);
                }(e, t, r, n);
            }
        }(e, t, r, n);
    }
}

function Rp(e, t, r, n) {
    return "function" == typeof xp.copyFileSync ? (xp.copyFileSync(t, r), xp.chmodSync(r, e.mode), 
    n.preserveTimestamps ? Pp(r, e.atime, e.mtime) : void 0) : function(e, t, r, n) {
        const o = 65536, i = (Op || (Op = 1, Ap = function(e) {
            if ("function" == typeof Buffer.allocUnsafe) {
                try {
                    return Buffer.allocUnsafe(e);
                } catch (t) {
                    return new Buffer(e);
                }
            }
            return new Buffer(e);
        }), Ap)(o), a = xp.openSync(t, "r"), s = xp.openSync(r, "w", e.mode);
        let u = 0;
        for (;u < e.size; ) {
            const e = xp.readSync(a, i, 0, o, u);
            xp.writeSync(s, i, 0, e), u += e;
        }
        n.preserveTimestamps && xp.futimesSync(s, e.atime, e.mtime);
        xp.closeSync(a), xp.closeSync(s);
    }(e, t, r, n);
}

function Tp(e, t, r) {
    xp.readdirSync(e).forEach(n => function(e, t, r, n) {
        const o = Fp.join(t, e), i = Fp.join(r, e), {destStat: a} = Ip.checkPathsSync(o, i, "copy");
        return kp(a, o, i, n);
    }(n, e, t, r));
}

var jp = function(e, t, r) {
    "function" == typeof r && (r = {
        filter: r
    }), (r = r || {}).clobber = !("clobber" in r) || !!r.clobber, r.overwrite = "overwrite" in r ? !!r.overwrite : r.clobber, 
    r.preserveTimestamps && "ia32" === process.arch && console.warn("fs-extra: Using the preserveTimestamps option in 32-bit node is not recommended;\n\n    see https://github.com/jprichardson/node-fs-extra/issues/269");
    const {srcStat: n, destStat: o} = Ip.checkPathsSync(e, t, "copy");
    return Ip.checkParentPathsSync(e, n, t, "copy"), function(e, t, r, n) {
        if (n.filter && !n.filter(t, r)) {
            return;
        }
        const o = Fp.dirname(r);
        xp.existsSync(o) || Mp(o);
        return kp(e, t, r, n);
    }(o, e, t, r);
}, Lp = {
    copySync: jp
};

const Np = Xd.fromPromise, Bp = Jd;

var Up = {
    pathExists: Np(function(e) {
        return Bp.access(e).then(() => !0).catch(() => !1);
    }),
    pathExistsSync: Bp.existsSync
};

const zp = li, Hp = r, $p = dp.mkdirs, Gp = Up.pathExists, Wp = hp, Vp = Cp;

function Kp(e, t, r, n, o) {
    const i = Hp.dirname(r);
    Gp(i, (a, s) => a ? o(a) : s ? Jp(e, t, r, n, o) : void $p(i, i => i ? o(i) : Jp(e, t, r, n, o)));
}

function qp(e, t, r, n, o, i) {
    Promise.resolve(o.filter(r, n)).then(a => a ? e(t, r, n, o, i) : i(), e => i(e));
}

function Jp(e, t, r, n, o) {
    return n.filter ? qp(Xp, e, t, r, n, o) : Xp(e, t, r, n, o);
}

function Xp(e, t, r, n, o) {
    (n.dereference ? zp.stat : zp.lstat)(t, (i, a) => i ? o(i) : a.isDirectory() ? function(e, t, r, n, o, i) {
        if (!t) {
            return function(e, t, r, n, o) {
                zp.mkdir(r, i => {
                    if (i) {
                        return o(i);
                    }
                    Qp(t, r, n, t => t ? o(t) : zp.chmod(r, e.mode, o));
                });
            }(e, r, n, o, i);
        }
        if (t && !t.isDirectory()) {
            return i(new Error(`Cannot overwrite non-directory '${n}' with directory '${r}'.`));
        }
        return Qp(r, n, o, i);
    }(a, e, t, r, n, o) : a.isFile() || a.isCharacterDevice() || a.isBlockDevice() ? function(e, t, r, n, o, i) {
        return t ? function(e, t, r, n, o) {
            if (!n.overwrite) {
                return n.errorOnExist ? o(new Error(`'${r}' already exists`)) : o();
            }
            zp.unlink(r, i => i ? o(i) : Zp(e, t, r, n, o));
        }(e, r, n, o, i) : Zp(e, r, n, o, i);
    }(a, e, t, r, n, o) : a.isSymbolicLink() ? function(e, t, r, n, o) {
        zp.readlink(t, (t, i) => t ? o(t) : (n.dereference && (i = Hp.resolve(process.cwd(), i)), 
        e ? void zp.readlink(r, (t, a) => t ? "EINVAL" === t.code || "UNKNOWN" === t.code ? zp.symlink(i, r, o) : o(t) : (n.dereference && (a = Hp.resolve(process.cwd(), a)), 
        Vp.isSrcSubdir(i, a) ? o(new Error(`Cannot copy '${i}' to a subdirectory of itself, '${a}'.`)) : e.isDirectory() && Vp.isSrcSubdir(a, i) ? o(new Error(`Cannot overwrite '${a}' with '${i}'.`)) : function(e, t, r) {
            zp.unlink(t, n => n ? r(n) : zp.symlink(e, t, r));
        }(i, r, o))) : zp.symlink(i, r, o)));
    }(e, t, r, n, o) : void 0);
}

function Zp(e, t, r, n, o) {
    return "function" == typeof zp.copyFile ? zp.copyFile(t, r, t => t ? o(t) : Yp(e, r, n, o)) : function(e, t, r, n, o) {
        const i = zp.createReadStream(t);
        i.on("error", e => o(e)).once("open", () => {
            const t = zp.createWriteStream(r, {
                mode: e.mode
            });
            t.on("error", e => o(e)).on("open", () => i.pipe(t)).once("close", () => Yp(e, r, n, o));
        });
    }(e, t, r, n, o);
}

function Yp(e, t, r, n) {
    zp.chmod(t, e.mode, o => o ? n(o) : r.preserveTimestamps ? Wp(t, e.atime, e.mtime, n) : n());
}

function Qp(e, t, r, n) {
    zp.readdir(e, (o, i) => o ? n(o) : eh(i, e, t, r, n));
}

function eh(e, t, r, n, o) {
    const i = e.pop();
    return i ? function(e, t, r, n, o, i) {
        const a = Hp.join(r, t), s = Hp.join(n, t);
        Vp.checkPaths(a, s, "copy", (t, u) => {
            if (t) {
                return i(t);
            }
            const {destStat: l} = u;
            Jp(l, a, s, o, t => t ? i(t) : eh(e, r, n, o, i));
        });
    }(e, i, t, r, n, o) : o();
}

var th = function(e, t, r, n) {
    "function" != typeof r || n ? "function" == typeof r && (r = {
        filter: r
    }) : (n = r, r = {}), n = n || function() {}, (r = r || {}).clobber = !("clobber" in r) || !!r.clobber, 
    r.overwrite = "overwrite" in r ? !!r.overwrite : r.clobber, r.preserveTimestamps && "ia32" === process.arch && console.warn("fs-extra: Using the preserveTimestamps option in 32-bit node is not recommended;\n\n    see https://github.com/jprichardson/node-fs-extra/issues/269"), 
    Vp.checkPaths(e, t, "copy", (o, i) => {
        if (o) {
            return n(o);
        }
        const {srcStat: a, destStat: s} = i;
        Vp.checkParentPaths(e, a, t, "copy", o => o ? n(o) : r.filter ? qp(Kp, s, e, t, r, n) : Kp(s, e, t, r, n));
    });
};

var rh = {
    copy: (0, Xd.fromCallback)(th)
};

const nh = li, oh = r, ih = c, ah = "win32" === process.platform;

function sh(e) {
    [ "unlink", "chmod", "stat", "lstat", "rmdir", "readdir" ].forEach(t => {
        e[t] = e[t] || nh[t], e[t += "Sync"] = e[t] || nh[t];
    }), e.maxBusyTries = e.maxBusyTries || 3;
}

function uh(e, t, r) {
    let n = 0;
    "function" == typeof t && (r = t, t = {}), ih(e, "rimraf: missing path"), ih.strictEqual(typeof e, "string", "rimraf: path should be a string"), 
    ih.strictEqual(typeof r, "function", "rimraf: callback function required"), ih(t, "rimraf: invalid options argument provided"), 
    ih.strictEqual(typeof t, "object", "rimraf: options should be object"), sh(t), lh(e, t, function o(i) {
        if (i) {
            if (("EBUSY" === i.code || "ENOTEMPTY" === i.code || "EPERM" === i.code) && n < t.maxBusyTries) {
                n++;
                return setTimeout(() => lh(e, t, o), 100 * n);
            }
            "ENOENT" === i.code && (i = null);
        }
        r(i);
    });
}

function lh(e, t, r) {
    ih(e), ih(t), ih("function" == typeof r), t.lstat(e, (n, o) => n && "ENOENT" === n.code ? r(null) : n && "EPERM" === n.code && ah ? ch(e, t, n, r) : o && o.isDirectory() ? dh(e, t, n, r) : void t.unlink(e, n => {
        if (n) {
            if ("ENOENT" === n.code) {
                return r(null);
            }
            if ("EPERM" === n.code) {
                return ah ? ch(e, t, n, r) : dh(e, t, n, r);
            }
            if ("EISDIR" === n.code) {
                return dh(e, t, n, r);
            }
        }
        return r(n);
    }));
}

function ch(e, t, r, n) {
    ih(e), ih(t), ih("function" == typeof n), r && ih(r instanceof Error), t.chmod(e, 438, o => {
        o ? n("ENOENT" === o.code ? null : r) : t.stat(e, (o, i) => {
            o ? n("ENOENT" === o.code ? null : r) : i.isDirectory() ? dh(e, t, r, n) : t.unlink(e, n);
        });
    });
}

function fh(e, t, r) {
    let n;
    ih(e), ih(t), r && ih(r instanceof Error);
    try {
        t.chmodSync(e, 438);
    } catch (e) {
        if ("ENOENT" === e.code) {
            return;
        }
        throw r;
    }
    try {
        n = t.statSync(e);
    } catch (e) {
        if ("ENOENT" === e.code) {
            return;
        }
        throw r;
    }
    n.isDirectory() ? hh(e, t, r) : t.unlinkSync(e);
}

function dh(e, t, r, n) {
    ih(e), ih(t), r && ih(r instanceof Error), ih("function" == typeof n), t.rmdir(e, o => {
        !o || "ENOTEMPTY" !== o.code && "EEXIST" !== o.code && "EPERM" !== o.code ? o && "ENOTDIR" === o.code ? n(r) : n(o) : function(e, t, r) {
            ih(e), ih(t), ih("function" == typeof r), t.readdir(e, (n, o) => {
                if (n) {
                    return r(n);
                }
                let i, a = o.length;
                if (0 === a) {
                    return t.rmdir(e, r);
                }
                o.forEach(n => {
                    uh(oh.join(e, n), t, n => {
                        if (!i) {
                            return n ? r(i = n) : void (0 === --a && t.rmdir(e, r));
                        }
                    });
                });
            });
        }(e, t, n);
    });
}

function ph(e, t) {
    let r;
    sh(t = t || {}), ih(e, "rimraf: missing path"), ih.strictEqual(typeof e, "string", "rimraf: path should be a string"), 
    ih(t, "rimraf: missing options"), ih.strictEqual(typeof t, "object", "rimraf: options should be object");
    try {
        r = t.lstatSync(e);
    } catch (r) {
        if ("ENOENT" === r.code) {
            return;
        }
        "EPERM" === r.code && ah && fh(e, t, r);
    }
    try {
        r && r.isDirectory() ? hh(e, t, null) : t.unlinkSync(e);
    } catch (r) {
        if ("ENOENT" === r.code) {
            return;
        }
        if ("EPERM" === r.code) {
            return ah ? fh(e, t, r) : hh(e, t, r);
        }
        if ("EISDIR" !== r.code) {
            throw r;
        }
        hh(e, t, r);
    }
}

function hh(e, t, r) {
    ih(e), ih(t), r && ih(r instanceof Error);
    try {
        t.rmdirSync(e);
    } catch (n) {
        if ("ENOTDIR" === n.code) {
            throw r;
        }
        if ("ENOTEMPTY" === n.code || "EEXIST" === n.code || "EPERM" === n.code) {
            !function(e, t) {
                if (ih(e), ih(t), t.readdirSync(e).forEach(r => ph(oh.join(e, r), t)), !ah) {
                    return t.rmdirSync(e, t);
                }
                {
                    const r = Date.now();
                    do {
                        try {
                            return t.rmdirSync(e, t);
                        } catch (e) {}
                    } while (Date.now() - r < 500);
                }
            }(e, t);
        } else if ("ENOENT" !== n.code) {
            throw n;
        }
    }
}

var vh = uh;

uh.sync = ph;

const gh = vh;

var mh = {
    remove: (0, Xd.fromCallback)(gh),
    removeSync: gh.sync
};

const yh = Xd.fromCallback, _h = li, Eh = r, bh = dp, wh = mh, Dh = yh(function(e, t) {
    t = t || function() {}, _h.readdir(e, (r, n) => {
        if (r) {
            return bh.mkdirs(e, t);
        }
        n = n.map(t => Eh.join(e, t)), function e() {
            const r = n.pop();
            if (!r) {
                return t();
            }
            wh.remove(r, r => {
                if (r) {
                    return t(r);
                }
                e();
            });
        }();
    });
});

function Sh(e) {
    let t;
    try {
        t = _h.readdirSync(e);
    } catch (t) {
        return bh.mkdirsSync(e);
    }
    t.forEach(t => {
        t = Eh.join(e, t), wh.removeSync(t);
    });
}

var Ah = {
    emptyDirSync: Sh,
    emptydirSync: Sh,
    emptyDir: Dh,
    emptydir: Dh
};

const Oh = Xd.fromCallback, Ch = r, xh = li, Fh = dp, Mh = Up.pathExists;

var Ph = {
    createFile: Oh(function(e, t) {
        function r() {
            xh.writeFile(e, "", e => {
                if (e) {
                    return t(e);
                }
                t();
            });
        }
        xh.stat(e, (n, o) => {
            if (!n && o.isFile()) {
                return t();
            }
            const i = Ch.dirname(e);
            Mh(i, (e, n) => e ? t(e) : n ? r() : void Fh.mkdirs(i, e => {
                if (e) {
                    return t(e);
                }
                r();
            }));
        });
    }),
    createFileSync: function(e) {
        let t;
        try {
            t = xh.statSync(e);
        } catch (e) {}
        if (t && t.isFile()) {
            return;
        }
        const r = Ch.dirname(e);
        xh.existsSync(r) || Fh.mkdirsSync(r), xh.writeFileSync(e, "");
    }
};

const Ih = Xd.fromCallback, kh = r, Rh = li, Th = dp, jh = Up.pathExists;

var Lh = {
    createLink: Ih(function(e, t, r) {
        function n(e, t) {
            Rh.link(e, t, e => {
                if (e) {
                    return r(e);
                }
                r(null);
            });
        }
        jh(t, (o, i) => o ? r(o) : i ? r(null) : void Rh.lstat(e, o => {
            if (o) {
                return o.message = o.message.replace("lstat", "ensureLink"), r(o);
            }
            const i = kh.dirname(t);
            jh(i, (o, a) => o ? r(o) : a ? n(e, t) : void Th.mkdirs(i, o => {
                if (o) {
                    return r(o);
                }
                n(e, t);
            }));
        }));
    }),
    createLinkSync: function(e, t) {
        if (Rh.existsSync(t)) {
            return;
        }
        try {
            Rh.lstatSync(e);
        } catch (e) {
            throw e.message = e.message.replace("lstat", "ensureLink"), e;
        }
        const r = kh.dirname(t);
        return Rh.existsSync(r) || Th.mkdirsSync(r), Rh.linkSync(e, t);
    }
};

const Nh = r, Bh = li, Uh = Up.pathExists;

var zh = {
    symlinkPaths: function(e, t, r) {
        if (Nh.isAbsolute(e)) {
            return Bh.lstat(e, t => t ? (t.message = t.message.replace("lstat", "ensureSymlink"), 
            r(t)) : r(null, {
                toCwd: e,
                toDst: e
            }));
        }
        {
            const n = Nh.dirname(t), o = Nh.join(n, e);
            return Uh(o, (t, i) => t ? r(t) : i ? r(null, {
                toCwd: o,
                toDst: e
            }) : Bh.lstat(e, t => t ? (t.message = t.message.replace("lstat", "ensureSymlink"), 
            r(t)) : r(null, {
                toCwd: e,
                toDst: Nh.relative(n, e)
            })));
        }
    },
    symlinkPathsSync: function(e, t) {
        let r;
        if (Nh.isAbsolute(e)) {
            if (r = Bh.existsSync(e), !r) {
                throw new Error("absolute srcpath does not exist");
            }
            return {
                toCwd: e,
                toDst: e
            };
        }
        {
            const n = Nh.dirname(t), o = Nh.join(n, e);
            if (r = Bh.existsSync(o), r) {
                return {
                    toCwd: o,
                    toDst: e
                };
            }
            if (r = Bh.existsSync(e), !r) {
                throw new Error("relative srcpath does not exist");
            }
            return {
                toCwd: e,
                toDst: Nh.relative(n, e)
            };
        }
    }
};

const Hh = li;

var $h = {
    symlinkType: function(e, t, r) {
        if (r = "function" == typeof t ? t : r, t = "function" != typeof t && t) {
            return r(null, t);
        }
        Hh.lstat(e, (e, n) => {
            if (e) {
                return r(null, "file");
            }
            t = n && n.isDirectory() ? "dir" : "file", r(null, t);
        });
    },
    symlinkTypeSync: function(e, t) {
        let r;
        if (t) {
            return t;
        }
        try {
            r = Hh.lstatSync(e);
        } catch (e) {
            return "file";
        }
        return r && r.isDirectory() ? "dir" : "file";
    }
};

const Gh = Xd.fromCallback, Wh = r, Vh = li, Kh = dp.mkdirs, qh = dp.mkdirsSync, Jh = zh.symlinkPaths, Xh = zh.symlinkPathsSync, Zh = $h.symlinkType, Yh = $h.symlinkTypeSync, Qh = Up.pathExists;

var ev = {
    createSymlink: Gh(function(e, t, r, n) {
        n = "function" == typeof r ? r : n, r = "function" != typeof r && r, Qh(t, (o, i) => o ? n(o) : i ? n(null) : void Jh(e, t, (o, i) => {
            if (o) {
                return n(o);
            }
            e = i.toDst, Zh(i.toCwd, r, (r, o) => {
                if (r) {
                    return n(r);
                }
                const i = Wh.dirname(t);
                Qh(i, (r, a) => r ? n(r) : a ? Vh.symlink(e, t, o, n) : void Kh(i, r => {
                    if (r) {
                        return n(r);
                    }
                    Vh.symlink(e, t, o, n);
                }));
            });
        }));
    }),
    createSymlinkSync: function(e, t, r) {
        if (Vh.existsSync(t)) {
            return;
        }
        const n = Xh(e, t);
        e = n.toDst, r = Yh(n.toCwd, r);
        const o = Wh.dirname(t);
        return Vh.existsSync(o) || qh(o), Vh.symlinkSync(e, t, r);
    }
};

var tv, rv = {
    createFile: Ph.createFile,
    createFileSync: Ph.createFileSync,
    ensureFile: Ph.createFile,
    ensureFileSync: Ph.createFileSync,
    createLink: Lh.createLink,
    createLinkSync: Lh.createLinkSync,
    ensureLink: Lh.createLink,
    ensureLinkSync: Lh.createLinkSync,
    createSymlink: ev.createSymlink,
    createSymlinkSync: ev.createSymlinkSync,
    ensureSymlink: ev.createSymlink,
    ensureSymlinkSync: ev.createSymlinkSync
};

try {
    tv = li;
} catch (Bk) {
    tv = t;
}

function nv(e, t) {
    var r, n = "\n";
    return "object" == typeof t && null !== t && (t.spaces && (r = t.spaces), t.EOL && (n = t.EOL)), 
    JSON.stringify(e, t ? t.replacer : null, r).replace(/\n/g, n) + n;
}

function ov(e) {
    return Buffer.isBuffer(e) && (e = e.toString("utf8")), e = e.replace(/^\uFEFF/, "");
}

var iv = {
    readFile: function(e, t, r) {
        null == r && (r = t, t = {}), "string" == typeof t && (t = {
            encoding: t
        });
        var n = (t = t || {}).fs || tv, o = !0;
        "throws" in t && (o = t.throws), n.readFile(e, t, function(n, i) {
            if (n) {
                return r(n);
            }
            var a;
            i = ov(i);
            try {
                a = JSON.parse(i, t ? t.reviver : null);
            } catch (t) {
                return o ? (t.message = e + ": " + t.message, r(t)) : r(null, null);
            }
            r(null, a);
        });
    },
    readFileSync: function(e, t) {
        "string" == typeof (t = t || {}) && (t = {
            encoding: t
        });
        var r = t.fs || tv, n = !0;
        "throws" in t && (n = t.throws);
        try {
            var o = r.readFileSync(e, t);
            return o = ov(o), JSON.parse(o, t.reviver);
        } catch (t) {
            if (n) {
                throw t.message = e + ": " + t.message, t;
            }
            return null;
        }
    },
    writeFile: function(e, t, r, n) {
        null == n && (n = r, r = {});
        var o = (r = r || {}).fs || tv, i = "";
        try {
            i = nv(t, r);
        } catch (e) {
            return void (n && n(e, null));
        }
        o.writeFile(e, i, r, n);
    },
    writeFileSync: function(e, t, r) {
        var n = (r = r || {}).fs || tv, o = nv(t, r);
        return n.writeFileSync(e, o, r);
    }
}, av = iv;

const sv = Xd.fromCallback, uv = av;

var lv = {
    readJson: sv(uv.readFile),
    readJsonSync: uv.readFileSync,
    writeJson: sv(uv.writeFile),
    writeJsonSync: uv.writeFileSync
};

const cv = r, fv = dp, dv = Up.pathExists, pv = lv;

var hv = function(e, t, r, n) {
    "function" == typeof r && (n = r, r = {});
    const o = cv.dirname(e);
    dv(o, (i, a) => i ? n(i) : a ? pv.writeJson(e, t, r, n) : void fv.mkdirs(o, o => {
        if (o) {
            return n(o);
        }
        pv.writeJson(e, t, r, n);
    }));
};

const vv = li, gv = r, mv = dp, yv = lv;

var _v = function(e, t, r) {
    const n = gv.dirname(e);
    vv.existsSync(n) || mv.mkdirsSync(n), yv.writeJsonSync(e, t, r);
};

const Ev = Xd.fromCallback, bv = lv;

bv.outputJson = Ev(hv), bv.outputJsonSync = _v, bv.outputJSON = bv.outputJson, bv.outputJSONSync = bv.outputJsonSync, 
bv.writeJSON = bv.writeJson, bv.writeJSONSync = bv.writeJsonSync, bv.readJSON = bv.readJson, 
bv.readJSONSync = bv.readJsonSync;

var wv = bv;

const Dv = li, Sv = r, Av = Lp.copySync, Ov = mh.removeSync, Cv = dp.mkdirpSync, xv = Cp;

function Fv(e, t, r) {
    try {
        Dv.renameSync(e, t);
    } catch (n) {
        if ("EXDEV" !== n.code) {
            throw n;
        }
        return function(e, t, r) {
            const n = {
                overwrite: r,
                errorOnExist: !0
            };
            return Av(e, t, n), Ov(e);
        }(e, t, r);
    }
}

var Mv = function(e, t, r) {
    const n = (r = r || {}).overwrite || r.clobber || !1, {srcStat: o} = xv.checkPathsSync(e, t, "move");
    return xv.checkParentPathsSync(e, o, t, "move"), Cv(Sv.dirname(t)), function(e, t, r) {
        if (r) {
            return Ov(t), Fv(e, t, r);
        }
        if (Dv.existsSync(t)) {
            throw new Error("dest already exists.");
        }
        return Fv(e, t, r);
    }(e, t, n);
}, Pv = {
    moveSync: Mv
};

const Iv = li, kv = r, Rv = rh.copy, Tv = mh.remove, jv = dp.mkdirp, Lv = Up.pathExists, Nv = Cp;

function Bv(e, t, r, n) {
    Iv.rename(e, t, o => o ? "EXDEV" !== o.code ? n(o) : function(e, t, r, n) {
        const o = {
            overwrite: r,
            errorOnExist: !0
        };
        Rv(e, t, o, t => t ? n(t) : Tv(e, n));
    }(e, t, r, n) : n());
}

var Uv = function(e, t, r, n) {
    "function" == typeof r && (n = r, r = {});
    const o = r.overwrite || r.clobber || !1;
    Nv.checkPaths(e, t, "move", (r, i) => {
        if (r) {
            return n(r);
        }
        const {srcStat: a} = i;
        Nv.checkParentPaths(e, a, t, "move", r => {
            if (r) {
                return n(r);
            }
            jv(kv.dirname(t), r => r ? n(r) : function(e, t, r, n) {
                if (r) {
                    return Tv(t, o => o ? n(o) : Bv(e, t, r, n));
                }
                Lv(t, (o, i) => o ? n(o) : i ? n(new Error("dest already exists.")) : Bv(e, t, r, n));
            }(e, t, o, n));
        });
    });
};

var zv = {
    move: (0, Xd.fromCallback)(Uv)
};

const Hv = Xd.fromCallback, $v = li, Gv = r, Wv = dp, Vv = Up.pathExists;

var Kv = {
    outputFile: Hv(function(e, t, r, n) {
        "function" == typeof r && (n = r, r = "utf8");
        const o = Gv.dirname(e);
        Vv(o, (i, a) => i ? n(i) : a ? $v.writeFile(e, t, r, n) : void Wv.mkdirs(o, o => {
            if (o) {
                return n(o);
            }
            $v.writeFile(e, t, r, n);
        }));
    }),
    outputFileSync: function(e, ...t) {
        const r = Gv.dirname(e);
        if ($v.existsSync(r)) {
            return $v.writeFileSync(e, ...t);
        }
        Wv.mkdirsSync(r), $v.writeFileSync(e, ...t);
    }
};

!function(e) {
    e.exports = Object.assign({}, Jd, Lp, rh, Ah, rv, wv, dp, Pv, zv, Kv, Up, mh);
    const r = t;
    Object.getOwnPropertyDescriptor(r, "promises") && Object.defineProperty(e.exports, "promises", {
        get: () => r.promises
    });
}(qd);

var qv = qd.exports;

const Jv = bf("streamroller:fileNameFormatter"), Xv = r;

const Zv = bf("streamroller:fileNameParser"), Yv = kf;

const Qv = bf("streamroller:moveAndMaybeCompressFile"), eg = qv, tg = p;

var rg = async (e, t, r) => {
    if (r = function(e) {
        const t = {
            mode: parseInt("0600", 8),
            compress: !1
        }, r = Object.assign({}, t, e);
        return Qv(`_parseOption: moveAndMaybeCompressFile called with option=${JSON.stringify(r)}`), 
        r;
    }(r), e !== t) {
        if (await eg.pathExists(e)) {
            if (Qv(`moveAndMaybeCompressFile: moving file from ${e} to ${t} ${r.compress ? "with" : "without"} compress`), 
            r.compress) {
                await new Promise((n, o) => {
                    let i = !1;
                    const a = eg.createWriteStream(t, {
                        mode: r.mode,
                        flags: "wx"
                    }).on("open", () => {
                        i = !0;
                        const t = eg.createReadStream(e).on("open", () => {
                            t.pipe(tg.createGzip()).pipe(a);
                        }).on("error", t => {
                            Qv(`moveAndMaybeCompressFile: error reading ${e}`, t), a.destroy(t);
                        });
                    }).on("finish", () => {
                        Qv(`moveAndMaybeCompressFile: finished compressing ${t}, deleting ${e}`), eg.unlink(e).then(n).catch(t => {
                            Qv(`moveAndMaybeCompressFile: error deleting ${e}, truncating instead`, t), eg.truncate(e).then(n).catch(t => {
                                Qv(`moveAndMaybeCompressFile: error truncating ${e}`, t), o(t);
                            });
                        });
                    }).on("error", e => {
                        i ? (Qv(`moveAndMaybeCompressFile: error writing ${t}, deleting`, e), eg.unlink(t).then(() => {
                            o(e);
                        }).catch(e => {
                            Qv(`moveAndMaybeCompressFile: error deleting ${t}`, e), o(e);
                        })) : (Qv(`moveAndMaybeCompressFile: error creating ${t}`, e), o(e));
                    });
                }).catch(() => {});
            } else {
                Qv(`moveAndMaybeCompressFile: renaming ${e} to ${t}`);
                try {
                    await eg.move(e, t, {
                        overwrite: !0
                    });
                } catch (r) {
                    if (Qv(`moveAndMaybeCompressFile: error renaming ${e} to ${t}`, r), "ENOENT" !== r.code) {
                        Qv("moveAndMaybeCompressFile: trying copy+truncate instead");
                        try {
                            await eg.copy(e, t, {
                                overwrite: !0
                            }), await eg.truncate(e);
                        } catch (e) {
                            Qv("moveAndMaybeCompressFile: error copy+truncate", e);
                        }
                    }
                }
            }
        }
    } else {
        Qv("moveAndMaybeCompressFile: source and target are the same, not doing anything");
    }
};

const ng = bf("streamroller:RollingFileWriteStream"), og = qv, ig = r, ag = a, sg = () => new Date, ug = kf, {Writable: lg} = u, cg = ({file: e, keepFileExt: t, needsIndex: r, alwaysIncludeDate: n, compress: o, fileNameSep: i}) => {
    let a = i || ".";
    const s = Xv.join(e.dir, e.name), u = t => t + e.ext, l = (e, t, n) => !r && n || !t ? e : e + a + t, c = (e, t, r) => (t > 0 || n) && r ? e + a + r : e, f = (e, t) => t && o ? e + ".gz" : e, d = t ? [ c, l, u, f ] : [ u, c, l, f ];
    return ({date: e, index: t}) => (Jv(`_formatFileName: date=${e}, index=${t}`), d.reduce((r, n) => n(r, t, e), s));
}, fg = ({file: e, keepFileExt: t, pattern: r, fileNameSep: n}) => {
    let o = n || ".";
    const i = "__NOT_MATCHING__";
    let a = [ (e, t) => e.endsWith(".gz") ? (Zv("it is gzipped"), t.isCompressed = !0, 
    e.slice(0, -3)) : e, t ? t => t.startsWith(e.name) && t.endsWith(e.ext) ? (Zv("it starts and ends with the right things"), 
    t.slice(e.name.length + 1, -1 * e.ext.length)) : i : t => t.startsWith(e.base) ? (Zv("it starts with the right things"), 
    t.slice(e.base.length + 1)) : i, r ? (e, t) => {
        const n = e.split(o);
        let i = n[n.length - 1];
        Zv("items: ", n, ", indexStr: ", i);
        let a = e;
        void 0 !== i && i.match(/^\d+$/) ? (a = e.slice(0, -1 * (i.length + 1)), Zv(`dateStr is ${a}`), 
        r && !a && (a = i, i = "0")) : i = "0";
        try {
            const n = Yv.parse(r, a, new Date(0, 0));
            return Yv.asString(r, n) !== a ? e : (t.index = parseInt(i, 10), t.date = a, t.timestamp = n.getTime(), 
            "");
        } catch (t) {
            return Zv(`Problem parsing ${a} as ${r}, error was: `, t), e;
        }
    } : (e, t) => e.match(/^\d+$/) ? (Zv("it has an index"), t.index = parseInt(e, 10), 
    "") : e ];
    return e => {
        let t = {
            filename: e,
            index: 0,
            isCompressed: !1
        };
        return a.reduce((e, r) => r(e, t), e) ? null : t;
    };
}, dg = rg;

var pg = class extends lg {
    constructor(e, t) {
        if (ng(`constructor: creating RollingFileWriteStream. path=${e}`), "string" != typeof e || 0 === e.length) {
            throw new Error(`Invalid filename: ${e}`);
        }
        if (e.endsWith(ig.sep)) {
            throw new Error(`Filename is a directory: ${e}`);
        }
        0 === e.indexOf(`~${ig.sep}`) && (e = e.replace("~", ag.homedir())), super(t), this.options = this._parseOption(t), 
        this.fileObject = ig.parse(e), "" === this.fileObject.dir && (this.fileObject = ig.parse(ig.join(process.cwd(), e))), 
        this.fileFormatter = cg({
            file: this.fileObject,
            alwaysIncludeDate: this.options.alwaysIncludePattern,
            needsIndex: this.options.maxSize < Number.MAX_SAFE_INTEGER,
            compress: this.options.compress,
            keepFileExt: this.options.keepFileExt,
            fileNameSep: this.options.fileNameSep
        }), this.fileNameParser = fg({
            file: this.fileObject,
            keepFileExt: this.options.keepFileExt,
            pattern: this.options.pattern,
            fileNameSep: this.options.fileNameSep
        }), this.state = {
            currentSize: 0
        }, this.options.pattern && (this.state.currentDate = ug(this.options.pattern, sg())), 
        this.filename = this.fileFormatter({
            index: 0,
            date: this.state.currentDate
        }), [ "a", "a+", "as", "as+" ].includes(this.options.flags) && this._setExistingSizeAndDate(), 
        ng(`constructor: create new file ${this.filename}, state=${JSON.stringify(this.state)}`), 
        this._renewWriteStream();
    }
    _setExistingSizeAndDate() {
        try {
            const e = og.statSync(this.filename);
            this.state.currentSize = e.size, this.options.pattern && (this.state.currentDate = ug(this.options.pattern, e.mtime));
        } catch (e) {
            return;
        }
    }
    _parseOption(e) {
        const t = {
            maxSize: 0,
            numToKeep: Number.MAX_SAFE_INTEGER,
            encoding: "utf8",
            mode: parseInt("0600", 8),
            flags: "a",
            compress: !1,
            keepFileExt: !1,
            alwaysIncludePattern: !1
        }, r = Object.assign({}, t, e);
        if (r.maxSize) {
            if (r.maxSize <= 0) {
                throw new Error(`options.maxSize (${r.maxSize}) should be > 0`);
            }
        } else {
            delete r.maxSize;
        }
        if (r.numBackups || 0 === r.numBackups) {
            if (r.numBackups < 0) {
                throw new Error(`options.numBackups (${r.numBackups}) should be >= 0`);
            }
            if (r.numBackups >= Number.MAX_SAFE_INTEGER) {
                throw new Error(`options.numBackups (${r.numBackups}) should be < Number.MAX_SAFE_INTEGER`);
            }
            r.numToKeep = r.numBackups + 1;
        } else if (r.numToKeep <= 0) {
            throw new Error(`options.numToKeep (${r.numToKeep}) should be > 0`);
        }
        return ng(`_parseOption: creating stream with option=${JSON.stringify(r)}`), r;
    }
    _final(e) {
        this.currentFileStream.end("", this.options.encoding, e);
    }
    _write(e, t, r) {
        this._shouldRoll().then(() => {
            ng(`_write: writing chunk. file=${this.currentFileStream.path} state=${JSON.stringify(this.state)} chunk=${e}`), 
            this.currentFileStream.write(e, t, t => {
                this.state.currentSize += e.length, r(t);
            });
        });
    }
    async _shouldRoll() {
        (this._dateChanged() || this._tooBig()) && (ng(`_shouldRoll: rolling because dateChanged? ${this._dateChanged()} or tooBig? ${this._tooBig()}`), 
        await this._roll());
    }
    _dateChanged() {
        return this.state.currentDate && this.state.currentDate !== ug(this.options.pattern, sg());
    }
    _tooBig() {
        return this.state.currentSize >= this.options.maxSize;
    }
    _roll() {
        return ng("_roll: closing the current stream"), new Promise((e, t) => {
            this.currentFileStream.end("", this.options.encoding, () => {
                this._moveOldFiles().then(e).catch(t);
            });
        });
    }
    async _moveOldFiles() {
        const e = await this._getExistingFiles();
        for (let t = (this.state.currentDate ? e.filter(e => e.date === this.state.currentDate) : e).length; t >= 0; t--) {
            ng(`_moveOldFiles: i = ${t}`);
            const e = this.fileFormatter({
                date: this.state.currentDate,
                index: t
            }), r = this.fileFormatter({
                date: this.state.currentDate,
                index: t + 1
            }), n = {
                compress: this.options.compress && 0 === t,
                mode: this.options.mode
            };
            await dg(e, r, n);
        }
        this.state.currentSize = 0, this.state.currentDate = this.state.currentDate ? ug(this.options.pattern, sg()) : null, 
        ng(`_moveOldFiles: finished rolling files. state=${JSON.stringify(this.state)}`), 
        this._renewWriteStream(), await new Promise((e, t) => {
            this.currentFileStream.write("", "utf8", () => {
                this._clean().then(e).catch(t);
            });
        });
    }
    async _getExistingFiles() {
        const e = await og.readdir(this.fileObject.dir).catch(() => []);
        ng(`_getExistingFiles: files=${e}`);
        const t = e.map(e => this.fileNameParser(e)).filter(e => e), r = e => (e.timestamp ? e.timestamp : sg().getTime()) - e.index;
        return t.sort((e, t) => r(e) - r(t)), t;
    }
    _renewWriteStream() {
        const e = this.fileFormatter({
            date: this.state.currentDate,
            index: 0
        }), t = e => {
            try {
                return og.mkdirSync(e, {
                    recursive: !0
                });
            } catch (r) {
                if ("ENOENT" === r.code) {
                    return t(ig.dirname(e)), t(e);
                }
                if ("EEXIST" !== r.code && "EROFS" !== r.code) {
                    throw r;
                }
                try {
                    if (og.statSync(e).isDirectory()) {
                        return e;
                    }
                    throw r;
                } catch (e) {
                    throw r;
                }
            }
        };
        t(this.fileObject.dir);
        const r = {
            flags: this.options.flags,
            encoding: this.options.encoding,
            mode: this.options.mode
        };
        var n, o;
        og.appendFileSync(e, "", (n = {
            ...r
        }, o = "flags", n["flag"] = n[o], delete n[o], n)), this.currentFileStream = og.createWriteStream(e, r), 
        this.currentFileStream.on("error", e => {
            this.emit("error", e);
        });
    }
    async _clean() {
        const e = await this._getExistingFiles();
        if (ng(`_clean: numToKeep = ${this.options.numToKeep}, existingFiles = ${e.length}`), 
        ng("_clean: existing files are: ", e), this._tooManyFiles(e.length)) {
            const r = e.slice(0, e.length - this.options.numToKeep).map(e => ig.format({
                dir: this.fileObject.dir,
                base: e.filename
            }));
            await (t = r, ng(`deleteFiles: files to delete: ${t}`), Promise.all(t.map(e => og.unlink(e).catch(t => {
                ng(`deleteFiles: error when unlinking ${e}, ignoring. Error was ${t}`);
            }))));
        }
        var t;
    }
    _tooManyFiles(e) {
        return this.options.numToKeep > 0 && e > this.options.numToKeep;
    }
};

const hg = pg;

var vg = class extends hg {
    constructor(e, t, r, n) {
        n || (n = {}), t && (n.maxSize = t), n.numBackups || 0 === n.numBackups || (r || 0 === r || (r = 1), 
        n.numBackups = r), super(e, n), this.backups = n.numBackups, this.size = this.options.maxSize;
    }
    get theStream() {
        return this.currentFileStream;
    }
};

const gg = pg;

var mg = class extends gg {
    constructor(e, t, r) {
        t && "object" == typeof t && (r = t, t = null), r || (r = {}), t || (t = "yyyy-MM-dd"), 
        r.pattern = t, r.numBackups || 0 === r.numBackups ? r.daysToKeep = r.numBackups : (r.daysToKeep || 0 === r.daysToKeep ? process.emitWarning("options.daysToKeep is deprecated due to the confusion it causes when used together with file size rolling. Please use options.numBackups instead.", "DeprecationWarning", "streamroller-DEP0001") : r.daysToKeep = 1, 
        r.numBackups = r.daysToKeep), super(e, r), this.mode = this.options.mode;
    }
    get theStream() {
        return this.currentFileStream;
    }
}, yg = {
    RollingFileWriteStream: pg,
    RollingFileStream: vg,
    DateRollingFileStream: mg
};

const _g = bf("log4js:file"), Eg = r, bg = yg, wg = a, Dg = wg.EOL;

let Sg = !1;

const Ag = new Set;

function Og() {
    Ag.forEach(e => {
        e.sighupHandler();
    });
}

Kd.configure = function(e, t) {
    let r = t.basicLayout;
    return e.layout && (r = t.layout(e.layout.type, e.layout)), e.mode = e.mode || 384, 
    function(e, t, r, n, o, i) {
        if ("string" != typeof e || 0 === e.length) {
            throw new Error(`Invalid filename: ${e}`);
        }
        if (e.endsWith(Eg.sep)) {
            throw new Error(`Filename is a directory: ${e}`);
        }
        function a(e, t, r, n) {
            const o = new bg.RollingFileStream(e, t, r, n);
            return o.on("error", t => {
                console.error("log4js.fileAppender - Writing to file %s, error happened ", e, t);
            }), o.on("drain", () => {
                process.emit("log4js:pause", !1);
            }), o;
        }
        0 === e.indexOf(`~${Eg.sep}`) && (e = e.replace("~", wg.homedir())), e = Eg.normalize(e), 
        _g("Creating file appender (", e, ", ", r, ", ", n = n || 0 === n ? n : 5, ", ", o, ", ", i, ")");
        let s = a(e, r, n, o);
        const u = function(e) {
            if (s.writable) {
                if (!0 === o.removeColor) {
                    const t = /\x1b[[0-9;]*m/g;
                    e.data = e.data.map(e => "string" == typeof e ? e.replace(t, "") : e);
                }
                s.write(t(e, i) + Dg, "utf8") || process.emit("log4js:pause", !0);
            }
        };
        return u.reopen = function() {
            s.end(() => {
                s = a(e, r, n, o);
            });
        }, u.sighupHandler = function() {
            _g("SIGHUP handler called."), u.reopen();
        }, u.shutdown = function(e) {
            Ag.delete(u), 0 === Ag.size && Sg && (process.removeListener("SIGHUP", Og), Sg = !1), 
            s.end("", "utf-8", e);
        }, Ag.add(u), Sg || (process.on("SIGHUP", Og), Sg = !0), u;
    }(e.filename, r, e.maxLogSize, e.backups, e, e.timezoneOffset);
};

var Cg = {};

const xg = yg, Fg = a.EOL;

function Mg(e, t, r, n, o) {
    n.maxSize = n.maxLogSize;
    const i = function(e, t, r) {
        const n = new xg.DateRollingFileStream(e, t, r);
        return n.on("error", t => {
            console.error("log4js.dateFileAppender - Writing to file %s, error happened ", e, t);
        }), n.on("drain", () => {
            process.emit("log4js:pause", !1);
        }), n;
    }(e, t, n), a = function(e) {
        i.writable && (i.write(r(e, o) + Fg, "utf8") || process.emit("log4js:pause", !0));
    };
    return a.shutdown = function(e) {
        i.end("", "utf-8", e);
    }, a;
}

Cg.configure = function(e, t) {
    let r = t.basicLayout;
    return e.layout && (r = t.layout(e.layout.type, e.layout)), e.alwaysIncludePattern || (e.alwaysIncludePattern = !1), 
    e.mode = e.mode || 384, Mg(e.filename, e.pattern, r, e, e.timezoneOffset);
};

var Pg = {};

const Ig = bf("log4js:fileSync"), kg = r, Rg = t, Tg = a, jg = Tg.EOL;

function Lg(e, t) {
    const r = e => {
        try {
            return Rg.mkdirSync(e, {
                recursive: !0
            });
        } catch (t) {
            if ("ENOENT" === t.code) {
                return r(kg.dirname(e)), r(e);
            }
            if ("EEXIST" !== t.code && "EROFS" !== t.code) {
                throw t;
            }
            try {
                if (Rg.statSync(e).isDirectory()) {
                    return e;
                }
                throw t;
            } catch (e) {
                throw t;
            }
        }
    };
    r(kg.dirname(e)), Rg.appendFileSync(e, "", {
        mode: t.mode,
        flag: t.flags
    });
}

class Ng {
    constructor(e, t, r, n) {
        if (Ig("In RollingFileStream"), t < 0) {
            throw new Error(`maxLogSize (${t}) should be > 0`);
        }
        this.filename = e, this.size = t, this.backups = r, this.options = n, this.currentSize = 0, 
        this.currentSize = function(e) {
            let t = 0;
            try {
                t = Rg.statSync(e).size;
            } catch (t) {
                Lg(e, n);
            }
            return t;
        }(this.filename);
    }
    shouldRoll() {
        return Ig("should roll with current size %d, and max size %d", this.currentSize, this.size), 
        this.currentSize >= this.size;
    }
    roll(e) {
        const t = this, r = new RegExp(`^${kg.basename(e)}`);
        function n(e) {
            return r.test(e);
        }
        function o(t) {
            return parseInt(t.slice(`${kg.basename(e)}.`.length), 10) || 0;
        }
        function i(e, t) {
            return o(e) - o(t);
        }
        function a(r) {
            const n = o(r);
            if (Ig(`Index of ${r} is ${n}`), 0 === t.backups) {
                Rg.truncateSync(e, 0);
            } else if (n < t.backups) {
                try {
                    Rg.unlinkSync(`${e}.${n + 1}`);
                } catch (e) {}
                Ig(`Renaming ${r} -> ${e}.${n + 1}`), Rg.renameSync(kg.join(kg.dirname(e), r), `${e}.${n + 1}`);
            }
        }
        Ig("Rolling, rolling, rolling"), Ig("Renaming the old files"), Rg.readdirSync(kg.dirname(e)).filter(n).sort(i).reverse().forEach(a);
    }
    write(e, t) {
        const r = this;
        Ig("in write"), this.shouldRoll() && (this.currentSize = 0, this.roll(this.filename)), 
        Ig("writing the chunk to the file"), r.currentSize += e.length, Rg.appendFileSync(r.filename, e);
    }
}

Pg.configure = function(e, t) {
    let r = t.basicLayout;
    e.layout && (r = t.layout(e.layout.type, e.layout));
    const n = {
        flags: e.flags || "a",
        encoding: e.encoding || "utf8",
        mode: e.mode || 384
    };
    return function(e, t, r, n, o, i) {
        if ("string" != typeof e || 0 === e.length) {
            throw new Error(`Invalid filename: ${e}`);
        }
        if (e.endsWith(kg.sep)) {
            throw new Error(`Filename is a directory: ${e}`);
        }
        0 === e.indexOf(`~${kg.sep}`) && (e = e.replace("~", Tg.homedir())), e = kg.normalize(e), 
        Ig("Creating fileSync appender (", e, ", ", r, ", ", n = n || 0 === n ? n : 5, ", ", o, ", ", i, ")");
        const a = function(e, t, r) {
            let n;
            var i;
            return t ? n = new Ng(e, t, r, o) : (Lg(i = e, o), n = {
                write(e) {
                    Rg.appendFileSync(i, e);
                }
            }), n;
        }(e, r, n);
        return e => {
            a.write(t(e, i) + jg);
        };
    }(e.filename, r, e.maxLogSize, e.backups, n, e.timezoneOffset);
};

var Bg = {};

const Ug = bf("log4js:tcp"), zg = h;

Bg.configure = function(e, t) {
    Ug(`configure with config = ${e}`);
    let r = function(e) {
        return e.serialise();
    };
    return e.layout && (r = t.layout(e.layout.type, e.layout)), function(e, t) {
        let r = !1;
        const n = [];
        let o, i = 3, a = "__LOG4JS__";
        function s(e) {
            Ug("Writing log event to socket"), r = o.write(`${t(e)}${a}`, "utf8");
        }
        function u() {
            let e;
            for (Ug("emptying buffer"); e = n.shift(); ) {
                s(e);
            }
        }
        function l(e) {
            r ? s(e) : (Ug("buffering log event because it cannot write at the moment"), n.push(e));
        }
        return function t() {
            Ug(`appender creating socket to ${e.host || "localhost"}:${e.port || 5e3}`), a = `${e.endMsg || "__LOG4JS__"}`, 
            o = zg.createConnection(e.port || 5e3, e.host || "localhost"), o.on("connect", () => {
                Ug("socket connected"), u(), r = !0;
            }), o.on("drain", () => {
                Ug("drain event received, emptying buffer"), r = !0, u();
            }), o.on("timeout", o.end.bind(o)), o.on("error", e => {
                Ug("connection error", e), r = !1, u();
            }), o.on("close", t);
        }(), l.shutdown = function(e) {
            Ug("shutdown called"), n.length && i ? (Ug("buffer has items, waiting 100ms to empty"), 
            i -= 1, setTimeout(() => {
                l.shutdown(e);
            }, 100)) : (o.removeAllListeners("close"), o.end(e));
        }, l;
    }(e, r);
};

const Hg = r, $g = bf("log4js:appenders"), Gg = Pf, Wg = kd, Vg = ed, Kg = Xf, qg = Rd, Jg = new Map;

Jg.set("console", Nd), Jg.set("stdout", Ud), Jg.set("stderr", zd), Jg.set("logLevelFilter", Hd), 
Jg.set("categoryFilter", $d), Jg.set("noLogFilter", Wd), Jg.set("file", Kd), Jg.set("dateFile", Cg), 
Jg.set("fileSync", Pg), Jg.set("tcp", Bg);

const Xg = new Map, Zg = (e, t) => {
    let r;
    try {
        const t = `${e}.cjs`;
        r = require.resolve(t), $g("Loading module from ", t);
    } catch (t) {
        r = e, $g("Loading module from ", e);
    }
    try {
        return require(r);
    } catch (r) {
        return void Gg.throwExceptionIf(t, "MODULE_NOT_FOUND" !== r.code, `appender "${e}" could not be loaded (error was: ${r})`);
    }
}, Yg = new Set, Qg = (e, t) => {
    if (Xg.has(e)) {
        return Xg.get(e);
    }
    if (!t.appenders[e]) {
        return !1;
    }
    if (Yg.has(e)) {
        throw new Error(`Dependency loop detected for appender ${e}.`);
    }
    Yg.add(e), $g(`Creating appender ${e}`);
    const r = em(e, t);
    return Yg.delete(e), Xg.set(e, r), r;
}, em = (e, t) => {
    const r = t.appenders[e], n = r.type.configure ? r.type : ((e, t) => Jg.get(e) || Zg(`./${e}`, t) || Zg(e, t) || require.main && require.main.filename && Zg(Hg.join(Hg.dirname(require.main.filename), e), t) || Zg(Hg.join(process.cwd(), e), t))(r.type, t);
    return Gg.throwExceptionIf(t, Gg.not(n), `appender "${e}" is not valid (type "${r.type}" could not be found)`), 
    n.appender && (process.emitWarning(`Appender ${r.type} exports an appender function.`, "DeprecationWarning", "log4js-node-DEP0001"), 
    $g("[log4js-node-DEP0001]", `DEPRECATION: Appender ${r.type} exports an appender function.`)), 
    n.shutdown && (process.emitWarning(`Appender ${r.type} exports a shutdown function.`, "DeprecationWarning", "log4js-node-DEP0002"), 
    $g("[log4js-node-DEP0002]", `DEPRECATION: Appender ${r.type} exports a shutdown function.`)), 
    $g(`${e}: clustering.isMaster ? ${Wg.isMaster()}`), $g(`${e}: appenderModule is ${l.inspect(n)}`), 
    Wg.onlyOnMaster(() => ($g(`calling appenderModule.configure for ${e} / ${r.type}`), 
    n.configure(qg.modifyConfig(r), Kg, e => Qg(e, t), Vg)), () => {});
}, tm = e => {
    if (Xg.clear(), Yg.clear(), !e) {
        return;
    }
    const t = [];
    Object.values(e.categories).forEach(e => {
        t.push(...e.appenders);
    }), Object.keys(e.appenders).forEach(r => {
        (t.includes(r) || "tcp-server" === e.appenders[r].type || "multiprocess" === e.appenders[r].type) && Qg(r, e);
    });
}, rm = () => {
    tm();
};

rm(), Gg.addListener(e => {
    Gg.throwExceptionIf(e, Gg.not(Gg.anObject(e.appenders)), 'must have a property "appenders" of type object.');
    const t = Object.keys(e.appenders);
    Gg.throwExceptionIf(e, Gg.not(t.length), "must define at least one appender."), 
    t.forEach(t => {
        Gg.throwExceptionIf(e, Gg.not(e.appenders[t].type), `appender "${t}" is not valid (must be an object with property "type")`);
    });
}), Gg.addListener(tm), td.exports = Xg, td.exports.init = rm;

var nm = td.exports, om = {
    exports: {}
};

!function(e) {
    const t = bf("log4js:categories"), r = Pf, n = ed, o = nm, i = new Map;
    function a(e, t, r) {
        if (!1 === t.inherit) {
            return;
        }
        const n = r.lastIndexOf(".");
        if (n < 0) {
            return;
        }
        const o = r.slice(0, n);
        let i = e.categories[o];
        i || (i = {
            inherit: !0,
            appenders: []
        }), a(e, i, o), !e.categories[o] && i.appenders && i.appenders.length && i.level && (e.categories[o] = i), 
        t.appenders = t.appenders || [], t.level = t.level || i.level, i.appenders.forEach(e => {
            t.appenders.includes(e) || t.appenders.push(e);
        }), t.parent = i;
    }
    function s(e) {
        if (!e.categories) {
            return;
        }
        Object.keys(e.categories).forEach(t => {
            const r = e.categories[t];
            a(e, r, t);
        });
    }
    r.addPreProcessingListener(e => s(e)), r.addListener(e => {
        r.throwExceptionIf(e, r.not(r.anObject(e.categories)), 'must have a property "categories" of type object.');
        const t = Object.keys(e.categories);
        r.throwExceptionIf(e, r.not(t.length), "must define at least one category."), t.forEach(t => {
            const i = e.categories[t];
            r.throwExceptionIf(e, [ r.not(i.appenders), r.not(i.level) ], `category "${t}" is not valid (must be an object with properties "appenders" and "level")`), 
            r.throwExceptionIf(e, r.not(Array.isArray(i.appenders)), `category "${t}" is not valid (appenders must be an array of appender names)`), 
            r.throwExceptionIf(e, r.not(i.appenders.length), `category "${t}" is not valid (appenders must contain at least one appender name)`), 
            Object.prototype.hasOwnProperty.call(i, "enableCallStack") && r.throwExceptionIf(e, "boolean" != typeof i.enableCallStack, `category "${t}" is not valid (enableCallStack must be boolean type)`), 
            i.appenders.forEach(n => {
                r.throwExceptionIf(e, r.not(o.get(n)), `category "${t}" is not valid (appender "${n}" is not defined)`);
            }), r.throwExceptionIf(e, r.not(n.getLevel(i.level)), `category "${t}" is not valid (level "${i.level}" not recognised; valid levels are ${n.levels.join(", ")})`);
        }), r.throwExceptionIf(e, r.not(e.categories.default), 'must define a "default" category.');
    });
    const u = e => {
        if (i.clear(), !e) {
            return;
        }
        Object.keys(e.categories).forEach(r => {
            const a = e.categories[r], s = [];
            a.appenders.forEach(e => {
                s.push(o.get(e)), t(`Creating category ${r}`), i.set(r, {
                    appenders: s,
                    level: n.getLevel(a.level),
                    enableCallStack: a.enableCallStack || !1
                });
            });
        });
    }, l = () => {
        u();
    };
    l(), r.addListener(u);
    const c = e => {
        if (t(`configForCategory: searching for config for ${e}`), i.has(e)) {
            return t(`configForCategory: ${e} exists in config, returning it`), i.get(e);
        }
        let r;
        return e.indexOf(".") > 0 ? (t(`configForCategory: ${e} has hierarchy, cloning from parents`), 
        r = {
            ...c(e.slice(0, e.lastIndexOf(".")))
        }) : (i.has("default") || u({
            categories: {
                default: {
                    appenders: [ "out" ],
                    level: "OFF"
                }
            }
        }), t("configForCategory: cloning default category"), r = {
            ...i.get("default")
        }), i.set(e, r), r;
    };
    e.exports = i, e.exports = Object.assign(e.exports, {
        appendersForCategory: e => c(e).appenders,
        getLevelForCategory: e => c(e).level,
        setLevelForCategory: (e, t) => {
            c(e).level = t;
        },
        getEnableCallStackForCategory: e => !0 === c(e).enableCallStack,
        setEnableCallStackForCategory: (e, t) => {
            c(e).enableCallStack = t;
        },
        init: l
    });
}(om);

var im = om.exports;

const am = bf("log4js:logger"), sm = Ed, um = ed, lm = kd, cm = im, fm = Pf, dm = /^(?:\s*)at (?:(.+) \()?(?:([^(]+?):(\d+):(\d+))\)?$/;

function pm(e, t = 4) {
    try {
        const r = e.stack.split("\n").slice(t);
        if (!r.length) {
            return null;
        }
        const n = dm.exec(r[0]);
        if (n && 5 === n.length) {
            let e = "", t = "", o = "";
            return n[1] && "" !== n[1] && ([t, o] = n[1].replace(/[[\]]/g, "").split(" as "), 
            o = o || "", t.includes(".") && ([e, t] = t.split("."))), {
                fileName: n[2],
                lineNumber: parseInt(n[3], 10),
                columnNumber: parseInt(n[4], 10),
                callStack: r.join("\n"),
                className: e,
                functionName: t,
                functionAlias: o,
                callerName: n[1] || ""
            };
        }
        console.error("log4js.logger - defaultParseCallStack error");
    } catch (e) {
        console.error("log4js.logger - defaultParseCallStack error", e);
    }
    return null;
}

let hm = class {
    constructor(e) {
        if (!e) {
            throw new Error("No category provided.");
        }
        this.category = e, this.context = {}, this.callStackSkipIndex = 0, this.parseCallStack = pm, 
        am(`Logger created (${this.category}, ${this.level})`);
    }
    get level() {
        return um.getLevel(cm.getLevelForCategory(this.category), um.OFF);
    }
    set level(e) {
        cm.setLevelForCategory(this.category, um.getLevel(e, this.level));
    }
    get useCallStack() {
        return cm.getEnableCallStackForCategory(this.category);
    }
    set useCallStack(e) {
        cm.setEnableCallStackForCategory(this.category, !0 === e);
    }
    get callStackLinesToSkip() {
        return this.callStackSkipIndex;
    }
    set callStackLinesToSkip(e) {
        if ("number" != typeof e) {
            throw new TypeError("Must be a number");
        }
        if (e < 0) {
            throw new RangeError("Must be >= 0");
        }
        this.callStackSkipIndex = e;
    }
    log(e, ...t) {
        const r = um.getLevel(e);
        r ? this.isLevelEnabled(r) && this._log(r, t) : fm.validIdentifier(e) && t.length > 0 ? (this.log(um.WARN, "log4js:logger.log: valid log-level not found as first parameter given:", e), 
        this.log(um.INFO, `[${e}]`, ...t)) : this.log(um.INFO, e, ...t);
    }
    isLevelEnabled(e) {
        return this.level.isLessThanOrEqualTo(e);
    }
    _log(e, t) {
        am(`sending log data (${e}) to appenders`);
        const r = t.find(e => e instanceof Error);
        let n;
        if (this.useCallStack) {
            try {
                r && (n = this.parseCallStack(r, this.callStackSkipIndex + 1));
            } catch (e) {}
            n = n || this.parseCallStack(new Error, this.callStackSkipIndex + 3 + 1);
        }
        const o = new sm(this.category, e, t, this.context, n, r);
        lm.send(o);
    }
    addContext(e, t) {
        this.context[e] = t;
    }
    removeContext(e) {
        delete this.context[e];
    }
    clearContext() {
        this.context = {};
    }
    setParseCallStackFunction(e) {
        if ("function" == typeof e) {
            this.parseCallStack = e;
        } else {
            if (void 0 !== e) {
                throw new TypeError("Invalid type passed to setParseCallStackFunction");
            }
            this.parseCallStack = pm;
        }
    }
};

function vm(e) {
    const t = um.getLevel(e), r = t.toString().toLowerCase().replace(/_([a-z])/g, e => e[1].toUpperCase()), n = r[0].toUpperCase() + r.slice(1);
    hm.prototype[`is${n}Enabled`] = function() {
        return this.isLevelEnabled(t);
    }, hm.prototype[r] = function(...e) {
        this.log(t, ...e);
    };
}

um.levels.forEach(vm), fm.addListener(() => {
    um.levels.forEach(vm);
});

var gm = hm;

const mm = ed;

function ym(e) {
    return e.originalUrl || e.url;
}

function _m(e, t) {
    for (let r = 0; r < t.length; r++) {
        e = e.replace(t[r].token, t[r].replacement);
    }
    return e;
}

const Em = bf("log4js:recording"), bm = [];

function wm() {
    return bm.slice();
}

function Dm() {
    bm.length = 0;
}

var Sm = {
    configure: function() {
        return function(e) {
            Em(`received logEvent, number of events now ${bm.length + 1}`), Em("log event was ", e), 
            bm.push(e);
        };
    },
    replay: wm,
    playback: wm,
    reset: Dm,
    erase: Dm
};

const Am = bf("log4js:main"), Om = t, Cm = wf({
    proto: !0
}), xm = Pf, Fm = nm, Mm = im, Pm = gm, Im = kd, km = function(e, t) {
    t = "string" == typeof t || "function" == typeof t ? {
        format: t
    } : t || {};
    const r = e;
    let n = mm.getLevel(t.level, mm.INFO);
    const o = t.format || ':remote-addr - - ":method :url HTTP/:http-version" :status :content-length ":referrer" ":user-agent"';
    return (e, i, a) => {
        if (void 0 !== e._logging) {
            return a();
        }
        if ("function" != typeof t.nolog) {
            const r = function(e) {
                let t = null;
                if (e instanceof RegExp && (t = e), "string" == typeof e && (t = new RegExp(e)), 
                Array.isArray(e)) {
                    const r = e.map(e => e.source ? e.source : e);
                    t = new RegExp(r.join("|"));
                }
                return t;
            }(t.nolog);
            if (r && r.test(e.originalUrl)) {
                return a();
            }
        }
        if (r.isLevelEnabled(n) || "auto" === t.level) {
            const a = new Date, {writeHead: s} = i;
            e._logging = !0, i.writeHead = (e, t) => {
                i.writeHead = s, i.writeHead(e, t), i.__statusCode = e, i.__headers = t || {};
            };
            let u = !1;
            const l = () => {
                if (u) {
                    return;
                }
                if (u = !0, "function" == typeof t.nolog && !0 === t.nolog(e, i)) {
                    return void (e._logging = !1);
                }
                i.responseTime = new Date - a, i.statusCode && "auto" === t.level && (n = mm.INFO, 
                i.statusCode >= 300 && (n = mm.WARN), i.statusCode >= 400 && (n = mm.ERROR)), n = function(e, t, r) {
                    let n = t;
                    if (r) {
                        const t = r.find(t => {
                            let r = !1;
                            return r = t.from && t.to ? e >= t.from && e <= t.to : -1 !== t.codes.indexOf(e), 
                            r;
                        });
                        t && (n = mm.getLevel(t.level, n));
                    }
                    return n;
                }(i.statusCode, n, t.statusRules);
                const s = function(e, t, r) {
                    const n = [];
                    return n.push({
                        token: ":url",
                        replacement: ym(e)
                    }), n.push({
                        token: ":protocol",
                        replacement: e.protocol
                    }), n.push({
                        token: ":hostname",
                        replacement: e.hostname
                    }), n.push({
                        token: ":method",
                        replacement: e.method
                    }), n.push({
                        token: ":status",
                        replacement: t.__statusCode || t.statusCode
                    }), n.push({
                        token: ":response-time",
                        replacement: t.responseTime
                    }), n.push({
                        token: ":date",
                        replacement: (new Date).toUTCString()
                    }), n.push({
                        token: ":referrer",
                        replacement: e.headers.referer || e.headers.referrer || ""
                    }), n.push({
                        token: ":http-version",
                        replacement: `${e.httpVersionMajor}.${e.httpVersionMinor}`
                    }), n.push({
                        token: ":remote-addr",
                        replacement: e.headers["x-forwarded-for"] || e.ip || e._remoteAddress || e.socket && (e.socket.remoteAddress || e.socket.socket && e.socket.socket.remoteAddress)
                    }), n.push({
                        token: ":user-agent",
                        replacement: e.headers["user-agent"]
                    }), n.push({
                        token: ":content-length",
                        replacement: t.getHeader("content-length") || t.__headers && t.__headers["Content-Length"] || "-"
                    }), n.push({
                        token: /:req\[([^\]]+)]/g,
                        replacement: (t, r) => e.headers[r.toLowerCase()]
                    }), n.push({
                        token: /:res\[([^\]]+)]/g,
                        replacement: (e, r) => t.getHeader(r.toLowerCase()) || t.__headers && t.__headers[r]
                    }), (e => {
                        const t = e.concat();
                        for (let e = 0; e < t.length; ++e) {
                            for (let r = e + 1; r < t.length; ++r) {
                                t[e].token == t[r].token && t.splice(r--, 1);
                            }
                        }
                        return t;
                    })(r.concat(n));
                }(e, i, t.tokens || []);
                if (t.context && r.addContext("res", i), "function" == typeof o) {
                    const t = o(e, i, e => _m(e, s));
                    t && r.log(n, t);
                } else {
                    r.log(n, _m(o, s));
                }
                t.context && r.removeContext("res");
            };
            i.on("end", l), i.on("finish", l), i.on("error", l), i.on("close", l);
        }
        return a();
    };
}, Rm = Sm;

let Tm = !1;

function jm(e) {
    if (!Tm) {
        return;
    }
    Am("Received log event ", e);
    Mm.appendersForCategory(e.categoryName).forEach(t => {
        t(e);
    });
}

function Lm(e) {
    Tm && Nm();
    let t = e;
    return "string" == typeof t && (t = function(e) {
        Am(`Loading configuration from ${e}`);
        try {
            return JSON.parse(Om.readFileSync(e, "utf8"));
        } catch (t) {
            throw new Error(`Problem reading config from file "${e}". Error was ${t.message}`, t);
        }
    }(e)), Am(`Configuration is ${t}`), xm.configure(Cm(t)), Im.onMessage(jm), Tm = !0, 
    Bm;
}

function Nm(e = () => {}) {
    if ("function" != typeof e) {
        throw new TypeError("Invalid callback passed to shutdown");
    }
    Am("Shutdown called. Disabling all log writing."), Tm = !1;
    const t = Array.from(Fm.values());
    Fm.init(), Mm.init();
    const r = t.reduce((e, t) => t.shutdown ? e + 1 : e, 0);
    0 === r && (Am("No appenders with shutdown functions found."), e());
    let n, o = 0;
    function i(t) {
        n = n || t, o += 1, Am(`Appender shutdowns complete: ${o} / ${r}`), o >= r && (Am("All shutdown functions completed."), 
        e(n));
    }
    Am(`Found ${r} appenders with shutdown functions.`), t.filter(e => e.shutdown).forEach(e => e.shutdown(i));
}

const Bm = {
    getLogger: function(e) {
        return Tm || Lm(process.env.LOG4JS_CONFIG || {
            appenders: {
                out: {
                    type: "stdout"
                }
            },
            categories: {
                default: {
                    appenders: [ "out" ],
                    level: "OFF"
                }
            }
        }), new Pm(e || "default");
    },
    configure: Lm,
    isConfigured: function() {
        return Tm;
    },
    shutdown: Nm,
    connectLogger: km,
    levels: ed,
    addLayout: Xf.addLayout,
    recording: function() {
        return Rm;
    }
};

var Um, zm, Hm, $m, Gm = Bm, Wm = {}, Vm = {};

function Km() {
    return zm || (zm = 1, function(e) {
        var t = g && g.__importDefault || function(e) {
            return e && e.__esModule ? e : {
                default: e
            };
        };
        Object.defineProperty(e, "__esModule", {
            value: !0
        }), e.HvigorLoggerConfig = void 0;
        const o = Gm, i = t(r), a = (Um || (Um = 1, function(e) {
            var t = g && g.__importDefault || function(e) {
                return e && e.__esModule ? e : {
                    default: e
                };
            };
            Object.defineProperty(e, "__esModule", {
                value: !0
            }), e.LogPathUtil = void 0;
            const o = t(r), i = t(n), a = Km(), s = Ic, u = Cc;
            class l {
                static logFilePath() {
                    let e;
                    try {
                        e = l.getHvigorCacheDir();
                    } catch {
                        e = o.default.resolve(l.HVIGOR_PROJECT_ROOT_DIR, l.HVIGOR_USER_HOME_DIR_NAME);
                    }
                    return o.default.resolve(e, "./outputs/build-logs");
                }
                static getHvigorCacheDir(e) {
                    var t;
                    let r = void 0 !== i.default.env.config ? JSON.parse(i.default.env.config)[u.BUILD_CACHE_DIR] : null !== (t = a.HvigorLoggerConfig.getExtraConfig(u.BUILD_CACHE_DIR)) && void 0 !== t ? t : l.getCommandHvigorCacheDir();
                    const n = o.default.resolve(l.HVIGOR_PROJECT_ROOT_DIR, l.HVIGOR_USER_HOME_DIR_NAME);
                    return r || (r = a.HvigorLoggerConfig.getHvigorCacheDir(), r) ? o.default.isAbsolute(r) ? (e && !this.hvigorCacheDirHasLogged && (e.warn((new s.HvigorLogInfo).setLogLevel(u.LogLevel.WARN).setMessage("Please ensure no projects of the same name have the same custom hvigor data dir.")), 
                    this.hvigorCacheDirHasLogged = !0), o.default.resolve(r, o.default.basename(i.default.cwd()), u.HVIGOR_USER_HOME_DIR_NAME)) : (e && !this.hvigorCacheDirHasLogged && (e.warn((new s.HvigorLogInfo).setLogLevel(u.LogLevel.WARN).setMessage(`Invalid custom hvigor data dir:${r}`)), 
                    this.hvigorCacheDirHasLogged = !0), n) : n;
                }
                static getCommandHvigorCacheDir() {
                    return i.default.argv.forEach(e => {
                        e.startsWith(u.BUILD_CACHE_DIR) && (i.default.env.BUILD_CACHE_DIR = e.substring(e.indexOf("=") + 1));
                    }), i.default.env.BUILD_CACHE_DIR;
                }
            }
            e.LogPathUtil = l, l.HVIGOR_PROJECT_ROOT_DIR = i.default.cwd(), l.HVIGOR_USER_HOME_DIR_NAME = ".hvigor", 
            l.hvigorCacheDirHasLogged = !1;
        }(Vm)), Vm);
        class s {
            static setExtraConfig(e) {
                s.extraConfig = e;
            }
            static getExtraConfig(e) {
                return s.extraConfig.get(e);
            }
            static setHvigorCacheDir(e) {
                s.hvigorCacheDir = e;
            }
            static getHvigorCacheDir() {
                return s.hvigorCacheDir;
            }
            static updateConfiguration() {
                const e = s.configuration.appenders["debug-log-file"];
                return e && "filename" in e && (e.filename = i.default.resolve(a.LogPathUtil.logFilePath(), "build.log")), 
                s.configuration;
            }
            static setCategoriesLevel(e, t) {
                s.logLevel = e;
                const r = s.configuration.categories;
                for (const n in r) {
                    (null == t ? void 0 : t.includes(n)) || n.includes("file") || Object.prototype.hasOwnProperty.call(r, n) && (r[n].level = e.levelStr);
                }
            }
            static getLevel() {
                return s.logLevel;
            }
        }
        e.HvigorLoggerConfig = s, s.extraConfig = new Map, s.configuration = {
            appenders: {
                debug: {
                    type: "stdout",
                    layout: {
                        type: "pattern",
                        pattern: "[%d] > hvigor %p %c %[%m%]"
                    }
                },
                "debug-log-file": {
                    type: "file",
                    filename: i.default.resolve(a.LogPathUtil.logFilePath(), "build.log"),
                    maxLogSize: 2097152,
                    backups: 9,
                    encoding: "utf-8",
                    level: "debug"
                },
                info: {
                    type: "stdout",
                    layout: {
                        type: "pattern",
                        pattern: "[%d] > hvigor %[%m%]"
                    }
                },
                "no-pattern-info": {
                    type: "stdout",
                    layout: {
                        type: "pattern",
                        pattern: "%m"
                    }
                },
                wrong: {
                    type: "stderr",
                    layout: {
                        type: "pattern",
                        pattern: "[%d] > hvigor %[%p: %m%]"
                    }
                },
                "just-debug": {
                    type: "logLevelFilter",
                    appender: "debug",
                    level: "debug",
                    maxLevel: "debug"
                },
                "just-info": {
                    type: "logLevelFilter",
                    appender: "info",
                    level: "info",
                    maxLevel: "info"
                },
                "just-wrong": {
                    type: "logLevelFilter",
                    appender: "wrong",
                    level: "warn",
                    maxLevel: "error"
                }
            },
            categories: {
                default: {
                    appenders: [ "just-debug", "just-info", "just-wrong" ],
                    level: "debug"
                },
                "no-pattern-info": {
                    appenders: [ "no-pattern-info" ],
                    level: "info"
                },
                "debug-file": {
                    appenders: [ "debug-log-file" ],
                    level: "debug"
                }
            }
        }, s.logLevel = o.levels.DEBUG, s.setConfiguration = e => {
            s.configuration = e;
        }, s.getConfiguration = () => s.configuration;
    }(Wm)), Wm;
}

function qm() {
    return $m || ($m = 1, function(e) {
        Object.defineProperty(e, "__esModule", {
            value: !0
        }), e.GroupBase = void 0;
        const t = (Hm || (Hm = 1, function(e) {
            Object.defineProperty(e, "__esModule", {
                value: !0
            }), e.HvigorFileLogger = void 0;
            const t = Cc, r = Fc, n = Iy(), o = Pc, i = lf;
            e.HvigorFileLogger = class {
                constructor(e) {
                    this.subSystem = t.SubSystemEnum.UNKNOWN, this.hvigorErrorCommonAdapter = new o.HvigorErrorCommonAdapter, 
                    this.hvigorLogList = [], this.subSystem = e;
                }
                printError(e) {
                    e = r.ErrorUtil.getRealError(e);
                    const t = this.hvigorErrorCommonAdapter.convertErrorToLogInfo(e);
                    i.HvigorLoggerForFile.getLogger(this.subSystem).error(t);
                }
                printErrorAndExit(e) {
                    e = r.ErrorUtil.getRealError(e);
                    const t = this.hvigorErrorCommonAdapter.convertErrorToLogInfo(e);
                    i.HvigorLoggerForFile.getLogger(this.subSystem).errorAndExit(t);
                }
                printWarn(e) {
                    const t = this.hvigorErrorCommonAdapter.convertWarnToLogInfo(e);
                    i.HvigorLoggerForFile.getLogger(this.subSystem).warn(t);
                }
                printInfo(e) {
                    const t = this.hvigorErrorCommonAdapter.convertInfoToLogInfo(e);
                    i.HvigorLoggerForFile.getLogger(this.subSystem).info(t);
                }
                printDebug(e) {
                    const t = this.hvigorErrorCommonAdapter.convertDebugToLogInfo(e);
                    i.HvigorLoggerForFile.getLogger(this.subSystem).debug(t);
                }
                pushError(e) {
                    this.hvigorLogList.push(r.ErrorUtil.getRealError(e));
                }
                printAllError(e) {
                    let t = this.hvigorLogList;
                    e && (t = e);
                    for (let e = 0; e < (null == t ? void 0 : t.length); e++) {
                        this.printError(t[e]);
                    }
                    e || (this.hvigorLogList = []);
                }
                printMergedError(e, t) {
                    let r = new n.MergeErrorList(this.hvigorLogList, e, t).getMergedErrorList();
                    this.printAllError(r), this.hvigorLogList = [];
                }
            };
        }(uf)), uf);
        e.GroupBase = class {
            constructor(e, r) {
                this.log = new t.HvigorFileLogger("hvigor"), this.hvigorLogList = e, this.mergeKey = r;
            }
            groupByKey(e, t) {
                this.hvigorLogList || (this.log.printWarn("hvigorLogList is undefined. Auto create empty list."), 
                this.hvigorLogList = []), e[this.mergeKey] && (this.log.printWarn(`mergeKey [${this.mergeKey}] can not exits in merge option. Auto delete this key.`), 
                delete e[this.mergeKey]), this._groupByKey(this.hvigorLogList, e, t);
            }
        };
    }(sf)), sf;
}

!function(e) {
    var t = g && g.__createBinding || (Object.create ? function(e, t, r, n) {
        void 0 === n && (n = r);
        var o = Object.getOwnPropertyDescriptor(t, r);
        o && !("get" in o ? !t.__esModule : o.writable || o.configurable) || (o = {
            enumerable: !0,
            get: function() {
                return t[r];
            }
        }), Object.defineProperty(e, n, o);
    } : function(e, t, r, n) {
        void 0 === n && (n = r), e[n] = t[r];
    }), r = g && g.__setModuleDefault || (Object.create ? function(e, t) {
        Object.defineProperty(e, "default", {
            enumerable: !0,
            value: t
        });
    } : function(e, t) {
        e.default = t;
    }), n = g && g.__importStar || function(e) {
        if (e && e.__esModule) {
            return e;
        }
        var n = {};
        if (null != e) {
            for (var o in e) {
                "default" !== o && Object.prototype.hasOwnProperty.call(e, o) && t(n, e, o);
            }
        }
        return r(n, e), n;
    };
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.HvigorLoggerForFile = void 0;
    const o = n(Gm), i = Km();
    class a {
        constructor(e, t) {
            o.configure(i.HvigorLoggerConfig.updateConfiguration()), this.fileLogger = o.getLogger("debug-file");
        }
        static getLogger(e) {
            return e || (e = "undefined"), a.hvigorLoggerCache.has(e) || a.hvigorLoggerCache.set(e, new a(e)), 
            a.hvigorLoggerCache.get(e);
        }
        static getLoggerWithDurationId(e, t) {
            return e || (e = "undefined"), a.hvigorLoggerCache.has(e) || a.hvigorLoggerCache.set(e, new a(e, t)), 
            a.hvigorLoggerCache.get(e);
        }
        debug(e) {
            this.fileLogger.debug(e.getMessage());
        }
        info(e) {
            this.fileLogger.debug(e.getMessage());
        }
        warn(e) {
            this.fileLogger.warn(e.getMessage());
        }
        error(e) {
            this.fileLogger.error(e.getMessage());
        }
        errorAndExit(e) {
            throw new Error(e.getMessage());
        }
    }
    e.HvigorLoggerForFile = a, a.hvigorLoggerCache = new Map;
}(lf);

var Jm, Xm, Zm = {};

function Ym() {
    return Xm || (Xm = 1, function(e) {
        Object.defineProperty(e, "__esModule", {
            value: !0
        }), e.CreateGroupByCause = void 0;
        const t = (Jm || (Jm = 1, function(e) {
            Object.defineProperty(e, "__esModule", {
                value: !0
            }), e.GroupByCause = void 0;
            const t = Cc, r = qm(), n = Zm;
            class o extends r.GroupBase {
                constructor(e) {
                    super(e, t.MergeKeyEnum.CAUSE);
                }
                _groupByKey(e, r, o) {
                    n.MergeUtil.composeErrors(t.MergeKeyEnum.CAUSE, e, r, o);
                }
            }
            e.GroupByCause = o;
        }(af)), af);
        e.CreateGroupByCause = class {
            newGroupByKey(e) {
                return new t.GroupByCause(e);
            }
        };
    }(of)), of;
}

!function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.MergeUtil = void 0;
    const t = Cc;
    class r {
        static init() {
            this.compareFuncMap.set(t.MergeKeyEnum.CODE, "compareCode"), this.compareFuncMap.set(t.MergeKeyEnum.CAUSE, "compareCause"), 
            this.compareFuncMap.set(t.MergeKeyEnum.POSITION, "comparePosition"), this.compareFuncMap.set(t.MergeKeyEnum.SOLUTIONS, "compareSolutions"), 
            this.compareFuncMap.set(t.MergeKeyEnum.MORE_INFO, "compareMoreInfo");
        }
        static composeErrors(e, t, n, o) {
            r.compareFuncMap.size < 1 && r.init(), t.forEach(t => {
                if (0 === o.size) {
                    return void o.set(Symbol(), [ t ]);
                }
                let r = !1;
                o.forEach((o, i) => {
                    const a = this.compareFuncMap.get(e);
                    a && n[a] && n[a](o[0], t) && (r = !0, o.push(t));
                }), r || o.set(Symbol(), [ t ]);
            });
        }
    }
    e.MergeUtil = r, r.compareFuncMap = new Map;
}(Zm);

var Qm, ey, ty = {}, ry = {};

function ny() {
    return ey || (ey = 1, function(e) {
        Object.defineProperty(e, "__esModule", {
            value: !0
        }), e.CreateGroupByCode = void 0;
        const t = (Qm || (Qm = 1, function(e) {
            Object.defineProperty(e, "__esModule", {
                value: !0
            }), e.GroupByCode = void 0;
            const t = Cc, r = qm(), n = Zm;
            class o extends r.GroupBase {
                constructor(e) {
                    super(e, t.MergeKeyEnum.CODE);
                }
                _groupByKey(e, r, o) {
                    n.MergeUtil.composeErrors(t.MergeKeyEnum.CODE, e, r, o);
                }
            }
            e.GroupByCode = o;
        }(ry)), ry);
        e.CreateGroupByCode = class {
            newGroupByKey(e) {
                return new t.GroupByCode(e);
            }
        };
    }(ty)), ty;
}

var oy, iy, ay = {}, sy = {};

function uy() {
    return iy || (iy = 1, function(e) {
        Object.defineProperty(e, "__esModule", {
            value: !0
        }), e.CreateGroupByMoreInfo = void 0;
        const t = (oy || (oy = 1, function(e) {
            Object.defineProperty(e, "__esModule", {
                value: !0
            }), e.GroupByMoreInfo = void 0;
            const t = Cc, r = qm(), n = Zm;
            class o extends r.GroupBase {
                constructor(e) {
                    super(e, t.MergeKeyEnum.MORE_INFO);
                }
                _groupByKey(e, r, o) {
                    n.MergeUtil.composeErrors(t.MergeKeyEnum.MORE_INFO, e, r, o);
                }
            }
            e.GroupByMoreInfo = o;
        }(sy)), sy);
        e.CreateGroupByMoreInfo = class {
            newGroupByKey(e) {
                return new t.GroupByMoreInfo(e);
            }
        };
    }(ay)), ay;
}

var ly, cy, fy = {}, dy = {};

function py() {
    return cy || (cy = 1, function(e) {
        Object.defineProperty(e, "__esModule", {
            value: !0
        }), e.CreateGroupByPosition = void 0;
        const t = (ly || (ly = 1, function(e) {
            Object.defineProperty(e, "__esModule", {
                value: !0
            }), e.GroupByPosition = void 0;
            const t = Cc, r = qm(), n = Zm;
            class o extends r.GroupBase {
                constructor(e) {
                    super(e, t.MergeKeyEnum.POSITION);
                }
                _groupByKey(e, r, o) {
                    n.MergeUtil.composeErrors(t.MergeKeyEnum.POSITION, e, r, o);
                }
            }
            e.GroupByPosition = o;
        }(dy)), dy);
        e.CreateGroupByPosition = class {
            newGroupByKey(e) {
                return new t.GroupByPosition(e);
            }
        };
    }(fy)), fy;
}

var hy, vy, gy = {}, my = {};

function yy() {
    return vy || (vy = 1, function(e) {
        Object.defineProperty(e, "__esModule", {
            value: !0
        }), e.CreateGroupBySolutions = void 0;
        const t = (hy || (hy = 1, function(e) {
            Object.defineProperty(e, "__esModule", {
                value: !0
            }), e.GroupBySolutions = void 0;
            const t = Cc, r = qm(), n = Zm;
            class o extends r.GroupBase {
                constructor(e) {
                    super(e, t.MergeKeyEnum.SOLUTIONS);
                }
                _groupByKey(e, r, o) {
                    n.MergeUtil.composeErrors(t.MergeKeyEnum.SOLUTIONS, e, r, o);
                }
            }
            e.GroupBySolutions = o;
        }(my)), my);
        e.CreateGroupBySolutions = class {
            newGroupByKey(e) {
                return new t.GroupBySolutions(e);
            }
        };
    }(gy)), gy;
}

var _y, Ey = {};

var by, wy = {};

var Dy, Sy = {};

var Ay, Oy = {};

var Cy, xy = {};

var Fy, My, Py = {};

function Iy() {
    return My || (My = 1, function(e) {
        Object.defineProperty(e, "__esModule", {
            value: !0
        }), e.MergeErrorList = void 0;
        const t = Cc, r = kc, n = Ym(), o = ny(), i = uy(), a = py(), s = yy(), u = (_y || (_y = 1, 
        function(e) {
            Object.defineProperty(e, "__esModule", {
                value: !0
            }), e.MergeCause = void 0;
            const t = Cc, r = jy(), n = kc;
            e.MergeCause = class {
                constructor() {
                    this.log = new r.HvigorConsoleLogger("hvigor");
                }
                mergeTwoKey(e, r, o) {
                    let i = "";
                    return e || r ? (e || (e = new n.HvigorErrorInfo), r || (r = new n.HvigorErrorInfo), 
                    o.cause === t.MergeType.COLLECT_ALL ? i = e.getCause() + t.SPLIT_TAG + r.getCause() : o.cause === t.MergeType.COLLECT_FIRST ? i = e.getCause() : o.cause === t.MergeType.COLLECT_LAST ? i = r.getCause() : (this.log.printDebug(`Unknown MergeType: ${o.code}. Use default strategy [COLLECT_LAST].`), 
                    i = r.getCode()), i) : i;
                }
            };
        }(Ey)), Ey), l = (by || (by = 1, function(e) {
            Object.defineProperty(e, "__esModule", {
                value: !0
            }), e.MergeCode = void 0;
            const t = Cc, r = jy(), n = kc;
            e.MergeCode = class {
                constructor() {
                    this.log = new r.HvigorConsoleLogger("hvigor");
                }
                mergeTwoKey(e, r, o) {
                    let i = "";
                    return e || r ? (e || (e = new n.HvigorErrorInfo), r || (r = new n.HvigorErrorInfo), 
                    o.code === t.MergeType.COLLECT_ALL ? i = e.getCode() + " " + r.getCode() : o.code === t.MergeType.COLLECT_FIRST ? i = e.getCode() : (o.code === t.MergeType.COLLECT_LAST || this.log.printDebug(`Unknown MergeType: ${o.code}. Use default strategy [COLLECT_LAST].`), 
                    i = r.getCode()), i) : i;
                }
            };
        }(wy)), wy), c = (Dy || (Dy = 1, function(e) {
            Object.defineProperty(e, "__esModule", {
                value: !0
            }), e.MergeDescription = void 0;
            const t = Cc, r = jy(), n = kc;
            e.MergeDescription = class {
                constructor() {
                    this.log = new r.HvigorConsoleLogger("hvigor");
                }
                mergeTwoKey(e, r, o) {
                    let i = "";
                    return e || r ? (e || (e = new n.HvigorErrorInfo), r || (r = new n.HvigorErrorInfo), 
                    o.code === t.MergeType.COLLECT_ALL ? i = e.getDescription() + " " + r.getDescription() : o.code === t.MergeType.COLLECT_FIRST ? i = e.getDescription() : (o.code === t.MergeType.COLLECT_LAST || this.log.printDebug(`Unknown MergeType: ${o.code}. Use default strategy [COLLECT_LAST].`), 
                    i = r.getDescription()), i) : i;
                }
            };
        }(Sy)), Sy), f = (Ay || (Ay = 1, function(e) {
            Object.defineProperty(e, "__esModule", {
                value: !0
            }), e.MergeMoreInfo = void 0;
            const t = Cc, r = jy(), n = kc;
            e.MergeMoreInfo = class {
                constructor() {
                    this.log = new r.HvigorConsoleLogger("hvigor");
                }
                mergeTwoKey(e, r, o) {
                    var i, a, s, u, l, c, f, d, p, h;
                    let v;
                    if (t.DEFAULT_MORE_INFO_URL_CN && t.DEFAULT_MORE_INFO_URL_EN) {
                        return e || r ? (e || (e = new n.HvigorErrorInfo), r || (r = new n.HvigorErrorInfo), 
                        o.moreInfo === t.MergeType.COLLECT_ALL ? v = {
                            cn: (null === (i = e.getMoreInfo()) || void 0 === i ? void 0 : i.cn) + " " + (null === (a = r.getMoreInfo()) || void 0 === a ? void 0 : a.cn),
                            en: (null === (s = e.getMoreInfo()) || void 0 === s ? void 0 : s.en) + " " + (null === (u = r.getMoreInfo()) || void 0 === u ? void 0 : u.en)
                        } : o.moreInfo === t.MergeType.COLLECT_FIRST ? v = {
                            cn: (null === (l = e.getMoreInfo()) || void 0 === l ? void 0 : l.cn) || t.DEFAULT_MORE_INFO_URL_CN,
                            en: (null === (c = e.getMoreInfo()) || void 0 === c ? void 0 : c.en) || t.DEFAULT_MORE_INFO_URL_EN
                        } : o.moreInfo === t.MergeType.COLLECT_LAST ? v = {
                            cn: (null === (f = r.getMoreInfo()) || void 0 === f ? void 0 : f.cn) || t.DEFAULT_MORE_INFO_URL_CN,
                            en: (null === (d = r.getMoreInfo()) || void 0 === d ? void 0 : d.en) || t.DEFAULT_MORE_INFO_URL_EN
                        } : (this.log.printDebug(`Unknown MergeType: ${o.code}. Use default strategy [COLLECT_LAST].`), 
                        v = {
                            cn: (null === (p = r.getMoreInfo()) || void 0 === p ? void 0 : p.cn) || t.DEFAULT_MORE_INFO_URL_CN,
                            en: (null === (h = r.getMoreInfo()) || void 0 === h ? void 0 : h.en) || t.DEFAULT_MORE_INFO_URL_EN
                        }), v) : {
                            cn: t.DEFAULT_MORE_INFO_URL_CN,
                            en: t.DEFAULT_MORE_INFO_URL_EN
                        };
                    }
                }
            };
        }(Oy)), Oy), d = (Cy || (Cy = 1, function(e) {
            Object.defineProperty(e, "__esModule", {
                value: !0
            }), e.MergePosition = void 0;
            const t = Cc, r = jy(), n = kc;
            e.MergePosition = class {
                constructor() {
                    this.log = new r.HvigorConsoleLogger("hvigor");
                }
                mergeTwoKey(e, r, o) {
                    let i = "";
                    return e || r ? (e || (e = new n.HvigorErrorInfo), r || (r = new n.HvigorErrorInfo), 
                    o.position === t.MergeType.COLLECT_ALL ? i = e.getPosition() + t.SPLIT_TAG + r.getPosition() : o.position === t.MergeType.COLLECT_FIRST ? i = e.getPosition() : o.position === t.MergeType.COLLECT_LAST ? i = r.getPosition() : (this.log.printDebug(`Unknown MergeType: ${o.code}. Use default strategy [COLLECT_LAST].`), 
                    i = r.getCode()), i) : i;
                }
            };
        }(xy)), xy), p = (Fy || (Fy = 1, function(e) {
            Object.defineProperty(e, "__esModule", {
                value: !0
            }), e.MergeSolutions = void 0;
            const t = Cc, r = jy(), n = kc;
            e.MergeSolutions = class {
                constructor() {
                    this.log = new r.HvigorConsoleLogger("hvigor");
                }
                mergeTwoKey(e, r, o) {
                    let i = [];
                    return e || r ? (e || (e = new n.HvigorErrorInfo), r || (r = new n.HvigorErrorInfo), 
                    o.solutions === t.MergeType.COLLECT_ALL ? i = [ ...e.getSolutions(), ...r.getSolutions() ] : o.solutions === t.MergeType.COLLECT_FIRST ? i = [ ...e.getSolutions() ] : (o.solutions === t.MergeType.COLLECT_LAST || this.log.printDebug(`Unknown MergeType: ${o.code}. Use default strategy [COLLECT_LAST].`), 
                    i = [ ...r.getSolutions() ]), i) : i;
                }
            };
        }(Py)), Py);
        class h {
            constructor(e, r, v) {
                this.hvigorLogList = e, this.mergeKey = r, this.option = this.handleOption(v), h.mergePropertyMap.set(t.MergeKeyEnum.CODE, new l.MergeCode), 
                h.mergePropertyMap.set("description", new c.MergeDescription), h.mergePropertyMap.set(t.MergeKeyEnum.CAUSE, new u.MergeCause), 
                h.mergePropertyMap.set(t.MergeKeyEnum.POSITION, new d.MergePosition), h.mergePropertyMap.set(t.MergeKeyEnum.SOLUTIONS, new p.MergeSolutions), 
                h.mergePropertyMap.set(t.MergeKeyEnum.MORE_INFO, new f.MergeMoreInfo), h.groupByKeyMap.set(t.MergeKeyEnum.CODE, new o.CreateGroupByCode), 
                h.groupByKeyMap.set(t.MergeKeyEnum.CAUSE, new n.CreateGroupByCause), h.groupByKeyMap.set(t.MergeKeyEnum.POSITION, new a.CreateGroupByPosition), 
                h.groupByKeyMap.set(t.MergeKeyEnum.SOLUTIONS, new s.CreateGroupBySolutions), h.groupByKeyMap.set(t.MergeKeyEnum.MORE_INFO, new i.CreateGroupByMoreInfo);
            }
            getMergedErrorList() {
                const e = this.getGroupedErrors(this.mergeKey, this.option);
                let t = [];
                return e.forEach((e, r) => {
                    t.push(this.mergeErrors(e, this.option));
                }), t;
            }
            getGroupedErrors(e, t) {
                var r;
                const n = new Map;
                return null === (r = h.groupByKeyMap.get(this.mergeKey)) || void 0 === r || r.newGroupByKey(this.hvigorLogList).groupByKey(t, n), 
                n;
            }
            mergeErrors(e, t) {
                let n = new r.HvigorErrorInfo({
                    code: e[0].getCode(),
                    cause: e[0].getCause(),
                    description: e[0].getDescription(),
                    position: e[0].getPosition(),
                    solutions: e[0].getSolutions(),
                    moreInfo: e[0].getMoreInfo()
                });
                for (let r = 1; r < e.length; r++) {
                    const o = e[r];
                    n = this.mergeTwoErrors(n, o, t);
                }
                return n;
            }
            mergeTwoErrors(e, t, n) {
                const o = this.handleOption(n), i = new r.HvigorErrorInfo;
                return h.mergePropertyMap.forEach((r, n) => {
                    i[n] = r.mergeTwoKey(e, t, o);
                }), i;
            }
            handleOption(e) {
                const r = {
                    code: t.MergeType.COLLECT_LAST,
                    cause: t.MergeType.COLLECT_LAST,
                    position: t.MergeType.COLLECT_LAST,
                    solutions: t.MergeType.COLLECT_LAST,
                    moreInfo: t.MergeType.COLLECT_LAST,
                    compareCode: (e, t) => e.getCode() === t.getCode(),
                    compareCause: (e, t) => e.getCause() === t.getCause(),
                    comparePosition: (e, t) => e.getPosition() === t.getPosition(),
                    compareMoreInfo: (e, t) => {
                        var r, n, o, i;
                        return !(!e || !t) && (null === (r = e.getMoreInfo()) || void 0 === r ? void 0 : r.cn) == (null === (n = t.getMoreInfo()) || void 0 === n ? void 0 : n.cn) && (null === (o = e.getMoreInfo()) || void 0 === o ? void 0 : o.en) == (null === (i = t.getMoreInfo()) || void 0 === i ? void 0 : i.en);
                    },
                    compareSolutions: (e, t) => {
                        if (!e || !t) {
                            return !1;
                        }
                        const r = e.getSolutions().sort(), n = t.getSolutions().sort();
                        return r.join() === n.join();
                    }
                };
                return Object.assign(r, e);
            }
        }
        e.MergeErrorList = h, h.mergePropertyMap = new Map, h.groupByKeyMap = new Map;
    }(nf)), nf;
}

var ky, Ry = {}, Ty = {};

function jy() {
    return ky || (ky = 1, function(e) {
        Object.defineProperty(e, "__esModule", {
            value: !0
        }), e.HvigorConsoleLogger = void 0;
        const t = Cc, r = Fc, n = Iy(), o = Pc, i = Ry;
        e.HvigorConsoleLogger = class {
            constructor(e) {
                this.subSystem = t.SubSystemEnum.UNKNOWN, this.hvigorErrorCommonAdapter = new o.HvigorErrorCommonAdapter, 
                this.hvigorLogList = [], this.subSystem = e;
            }
            printError(e) {
                e = r.ErrorUtil.getRealError(e);
                const t = this.hvigorErrorCommonAdapter.convertErrorToLogInfo(e);
                i.HvigorLoggerForConsole.getLogger(this.subSystem).error(t);
            }
            printErrorAndExit(e) {
                e = r.ErrorUtil.getRealError(e);
                const t = this.hvigorErrorCommonAdapter.convertErrorToLogInfo(e);
                i.HvigorLoggerForConsole.getLogger(this.subSystem).errorAndExit(t);
            }
            printWarn(e) {
                const t = this.hvigorErrorCommonAdapter.convertWarnToLogInfo(e);
                i.HvigorLoggerForConsole.getLogger(this.subSystem).warn(t);
            }
            printInfo(e) {
                const t = this.hvigorErrorCommonAdapter.convertInfoToLogInfo(e);
                i.HvigorLoggerForConsole.getLogger(this.subSystem).info(t);
            }
            printDebug(e) {
                const t = this.hvigorErrorCommonAdapter.convertDebugToLogInfo(e);
                i.HvigorLoggerForConsole.getLogger(this.subSystem).debug(t);
            }
            pushError(e) {
                this.hvigorLogList.push(r.ErrorUtil.getRealError(e));
            }
            printAllError(e) {
                let t = this.hvigorLogList;
                e && (t = e);
                for (let e = 0; e < (null == t ? void 0 : t.length); e++) {
                    this.printError(t[e]);
                }
                e || (this.hvigorLogList = []);
            }
            printMergedError(e, t) {
                let r = new n.MergeErrorList(this.hvigorLogList, e, t).getMergedErrorList();
                this.printAllError(r), this.hvigorLogList = [];
            }
        };
    }(rf)), rf;
}

!function(e) {
    var t = g && g.__createBinding || (Object.create ? function(e, t, r, n) {
        void 0 === n && (n = r);
        var o = Object.getOwnPropertyDescriptor(t, r);
        o && !("get" in o ? !t.__esModule : o.writable || o.configurable) || (o = {
            enumerable: !0,
            get: function() {
                return t[r];
            }
        }), Object.defineProperty(e, n, o);
    } : function(e, t, r, n) {
        void 0 === n && (n = r), e[n] = t[r];
    }), r = g && g.__setModuleDefault || (Object.create ? function(e, t) {
        Object.defineProperty(e, "default", {
            enumerable: !0,
            value: t
        });
    } : function(e, t) {
        e.default = t;
    }), n = g && g.__importStar || function(e) {
        if (e && e.__esModule) {
            return e;
        }
        var n = {};
        if (null != e) {
            for (var o in e) {
                "default" !== o && Object.prototype.hasOwnProperty.call(e, o) && t(n, e, o);
            }
        }
        return r(n, e), n;
    };
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.HvigorLogger = void 0;
    const o = n(Gm), i = Km();
    e.HvigorLogger = class {
        constructor(e, t) {
            o.configure(i.HvigorLoggerConfig.updateConfiguration()), this.consoleLogger = o.getLogger(e), 
            this.setLevel(i.HvigorLoggerConfig.getLevel()), this.fileLogger = o.getLogger("debug-file");
        }
        getFileLogger() {
            return this.fileLogger;
        }
        getConsoleLogger() {
            return this.consoleLogger;
        }
        debug(e) {
            void 0 !== e.getMessage() && "" !== e.getMessage() && (this.consoleLogger.debug(e.getMessage()), 
            this.fileLogger.debug(e.getMessage()));
        }
        info(e) {
            void 0 !== e.getMessage() && "" !== e.getMessage() && (this.consoleLogger.info(e.getMessage()), 
            this.fileLogger.info(e.getMessage()));
        }
        warn(e) {
            void 0 !== e.getMessage() && "" !== e.getMessage() && (this.consoleLogger.warn(e.getMessage()), 
            this.fileLogger.warn(e.getMessage()));
        }
        error(e) {
            this.consoleLogger.error(e.getMessage()), this.fileLogger.error(e.getMessage());
        }
        errorAndExit(e) {
            throw this.consoleLogger.error(e.getMessage()), this.fileLogger.error(e.getMessage()), 
            new Error(e.getMessage());
        }
        errorWithoutStack(e) {
            this.consoleLogger.error(e.getMessage()), process.exit(-1);
        }
        setLevel(e) {
            this.consoleLogger.level = e;
        }
    };
}(Ty), function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.HvigorLoggerForConsole = void 0;
    const t = Ty;
    class r extends t.HvigorLogger {
        constructor(e, t) {
            super(e, t);
        }
        static getLogger(e) {
            return e || (e = "undefined"), r.hvigorLoggerCache.has(e) || r.hvigorLoggerCache.set(e, new r(e)), 
            r.hvigorLoggerCache.get(e);
        }
        static getLoggerWithDurationId(e, t) {
            return e || (e = "undefined"), r.hvigorLoggerCache.has(e) || r.hvigorLoggerCache.set(e, new r(e, t)), 
            r.hvigorLoggerCache.get(e);
        }
    }
    e.HvigorLoggerForConsole = r, r.hvigorLoggerCache = new Map;
}(Ry);

var Ly = {};

Object.defineProperty(Ly, "__esModule", {
    value: !0
}), function(e) {
    var t = g && g.__createBinding || (Object.create ? function(e, t, r, n) {
        void 0 === n && (n = r);
        var o = Object.getOwnPropertyDescriptor(t, r);
        o && !("get" in o ? !t.__esModule : o.writable || o.configurable) || (o = {
            enumerable: !0,
            get: function() {
                return t[r];
            }
        }), Object.defineProperty(e, n, o);
    } : function(e, t, r, n) {
        void 0 === n && (n = r), e[n] = t[r];
    }), r = g && g.__exportStar || function(e, r) {
        for (var n in e) {
            "default" === n || Object.prototype.hasOwnProperty.call(r, n) || t(r, e, n);
        }
    };
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.DEVELOPER_URL = e.getUrl = e.errorCode2Description = e.formatErrorAdaptorExport = e.MergeErrorList = e.HvigorErrorCommonAdapter = e.ErrorUtil = e.HvigorLoggerConfig = e.HvigorErrorInfo = e.HvigorConsoleLogger = e.AdaptorError = e.MATCH_FIELD_TYPES = e.ThirdPartyErrorAdaptor = e.PackToolAdaptor = e.MixAdaptor = e.HvigorOhosPluginAdaptor = e.HvigorGlobalErrorAdaptor = e.HvigorErrorAdaptor = e.ArkTsErrorAdaptor = void 0;
    var n = Sc;
    Object.defineProperty(e, "ArkTsErrorAdaptor", {
        enumerable: !0,
        get: function() {
            return n.ArkTsErrorAdaptor;
        }
    });
    var o = zc();
    Object.defineProperty(e, "HvigorErrorAdaptor", {
        enumerable: !0,
        get: function() {
            return o.HvigorErrorAdaptor;
        }
    });
    var i = Wc;
    Object.defineProperty(e, "HvigorGlobalErrorAdaptor", {
        enumerable: !0,
        get: function() {
            return i.HvigorGlobalErrorAdaptor;
        }
    });
    var a = Vc;
    Object.defineProperty(e, "HvigorOhosPluginAdaptor", {
        enumerable: !0,
        get: function() {
            return a.HvigorOhosPluginAdaptor;
        }
    });
    var s = Kc;
    Object.defineProperty(e, "MixAdaptor", {
        enumerable: !0,
        get: function() {
            return s.MixAdaptor;
        }
    });
    var u = Xc;
    Object.defineProperty(e, "PackToolAdaptor", {
        enumerable: !0,
        get: function() {
            return u.PackToolAdaptor;
        }
    });
    var l = $c;
    Object.defineProperty(e, "ThirdPartyErrorAdaptor", {
        enumerable: !0,
        get: function() {
            return l.ThirdPartyErrorAdaptor;
        }
    });
    var c = jc;
    Object.defineProperty(e, "MATCH_FIELD_TYPES", {
        enumerable: !0,
        get: function() {
            return c.MATCH_FIELD_TYPES;
        }
    });
    var f = Mc;
    Object.defineProperty(e, "AdaptorError", {
        enumerable: !0,
        get: function() {
            return f.AdaptorError;
        }
    });
    var d = jy();
    Object.defineProperty(e, "HvigorConsoleLogger", {
        enumerable: !0,
        get: function() {
            return d.HvigorConsoleLogger;
        }
    });
    var p = kc;
    Object.defineProperty(e, "HvigorErrorInfo", {
        enumerable: !0,
        get: function() {
            return p.HvigorErrorInfo;
        }
    });
    var h = Km();
    Object.defineProperty(e, "HvigorLoggerConfig", {
        enumerable: !0,
        get: function() {
            return h.HvigorLoggerConfig;
        }
    });
    var v = Fc;
    Object.defineProperty(e, "ErrorUtil", {
        enumerable: !0,
        get: function() {
            return v.ErrorUtil;
        }
    });
    var m = Pc;
    Object.defineProperty(e, "HvigorErrorCommonAdapter", {
        enumerable: !0,
        get: function() {
            return m.HvigorErrorCommonAdapter;
        }
    });
    var y = Iy();
    Object.defineProperty(e, "MergeErrorList", {
        enumerable: !0,
        get: function() {
            return y.MergeErrorList;
        }
    });
    var _ = Hc();
    Object.defineProperty(e, "formatErrorAdaptorExport", {
        enumerable: !0,
        get: function() {
            return _.formatErrorAdaptorExport;
        }
    }), Object.defineProperty(e, "errorCode2Description", {
        enumerable: !0,
        get: function() {
            return _.errorCode2Description;
        }
    }), r(Ly, e), r(Cc, e);
    var E = xc;
    Object.defineProperty(e, "getUrl", {
        enumerable: !0,
        get: function() {
            return E.getUrl;
        }
    }), Object.defineProperty(e, "DEVELOPER_URL", {
        enumerable: !0,
        get: function() {
            return E.DEVELOPER_URL;
        }
    });
}(Dc);

var Ny = {}, By = {}, Uy = {}, zy = {}, Hy = {}, $y = {};

Object.defineProperty($y, "__esModule", {
    value: !0
});

var Gy = Object.prototype.toString;

$y.default = function(e) {
    return null == e ? void 0 === e ? "[object Undefined]" : "[object Null]" : Gy.call(e);
};

var Wy = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Hy, "__esModule", {
    value: !0
}), Hy.isFlattenable = Hy.isArguments = Hy.baseIsNaN = Hy.isPrototype = Hy.isIterate = Hy.isArray = Hy.isLength = Hy.isEqual = Hy.isIndex = Hy.isObject = void 0;

var Vy = Wy($y);

function Ky(e) {
    var t = typeof e;
    return null != e && ("object" === t || "function" === t);
}

Hy.isObject = Ky;

var qy = /^(?:0|[1-9]\d*)$/;

function Jy(e, t) {
    var r = typeof e, n = t;
    return !!(n = null == n ? Number.MAX_SAFE_INTEGER : n) && ("number" === r || "symbol" !== r && qy.test(e)) && e > -1 && e % 1 == 0 && e < n;
}

function Xy(e, t) {
    return e === t || Number.isNaN(e) && Number.isNaN(t);
}

function Zy(e) {
    return "number" == typeof e && e > -1 && e % 1 == 0 && e <= Number.MAX_SAFE_INTEGER;
}

function Yy(e) {
    return null != e && "function" != typeof e && Zy(e.length);
}

Hy.isIndex = Jy, Hy.isEqual = Xy, Hy.isLength = Zy, Hy.isArray = Yy, Hy.isIterate = function(e, t, r) {
    if (!Ky(r)) {
        return !1;
    }
    var n = typeof t;
    return !!("number" === n ? Yy(r) && Jy(t, r.length) : "string" === n && t in r) && Xy(r[t], e);
};

var Qy = Object.prototype;

function e_(e) {
    return Ky(e) && "[object Arguments]" === (0, Vy.default)(e);
}

Hy.isPrototype = function(e) {
    var t = e && e.constructor;
    return e === ("function" == typeof t && t.prototype || Qy);
}, Hy.baseIsNaN = function(e) {
    return Number.isNaN(e);
}, Hy.isArguments = e_;

var t_ = Symbol.isConcatSpreadable;

Hy.isFlattenable = function(e) {
    return Array.isArray(e) || e_(e) || !(!e || !e[t_]);
};

var r_ = {}, n_ = {};

Object.defineProperty(n_, "__esModule", {
    value: !0
}), n_.assignValue = n_.baseAssignValue = void 0;

var o_ = Hy;

function i_(e, t, r) {
    "__proto__" === t ? Object.defineProperty(e, t, {
        configurable: !0,
        enumerable: !0,
        value: r,
        writable: !0
    }) : e[t] = r;
}

n_.baseAssignValue = i_;

var a_ = Object.prototype.hasOwnProperty;

n_.assignValue = function(e, t, r) {
    var n = e[t];
    a_.call(e, t) && (0, o_.isEqual)(n, r) && (void 0 !== r || t in e) || i_(e, t, r);
};

var s_ = g && g.__values || function(e) {
    var t = "function" == typeof Symbol && Symbol.iterator, r = t && e[t], n = 0;
    if (r) {
        return r.call(e);
    }
    if (e && "number" == typeof e.length) {
        return {
            next: function() {
                return e && n >= e.length && (e = void 0), {
                    value: e && e[n++],
                    done: !e
                };
            }
        };
    }
    throw new TypeError(t ? "Object is not iterable." : "Symbol.iterator is not defined.");
};

Object.defineProperty(r_, "__esModule", {
    value: !0
}), r_.copyObject = void 0;

var u_ = n_, l_ = Hy;

r_.copyObject = function(e, t, r, n) {
    var o, i, a = r, s = !a;
    (0, l_.isObject)(a) || (a = {});
    try {
        for (var u = s_(t), l = u.next(); !l.done; l = u.next()) {
            var c = l.value, f = n ? n(a[c], e[c], c, a, e) : void 0;
            void 0 === f && (f = e[c]), s ? (0, u_.baseAssignValue)(a, c, f) : (0, u_.assignValue)(a, c, f);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            l && !l.done && (i = u.return) && i.call(u);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
};

var c_ = {}, f_ = {};

Object.defineProperty(f_, "__esModule", {
    value: !0
});

var d_ = Hy;

f_.default = function(e) {
    return null != e && "function" != typeof e && (0, d_.isLength)(e.length);
};

var p_ = {}, h_ = {}, v_ = {};

Object.defineProperty(v_, "__esModule", {
    value: !0
}), v_.default = function(e) {
    return "object" == typeof e && null !== e;
};

var g_ = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(h_, "__esModule", {
    value: !0
});

var m_ = g_($y), y_ = g_(v_), __ = /^\[object (?:Float(?:32|64)|(?:Int|Uint)(?:8|16|32)|Uint8Clamped)Array\]$/;

h_.default = function(e) {
    return (0, y_.default)(e) && __.test((0, m_.default)(e));
};

var E_ = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(p_, "__esModule", {
    value: !0
});

var b_ = Hy, w_ = E_(h_), D_ = Object.prototype.hasOwnProperty;

p_.default = function(e, t) {
    for (var r = function(e, t) {
        var r = !e && (0, b_.isArguments)(t), n = !e && !r && !1, o = !e && !r && !n && (0, 
        w_.default)(t);
        return e || r || n || o;
    }(Array.isArray(e), e), n = e.length, o = new Array(r ? n : 0), i = r ? -1 : n; ++i < n; ) {
        o[i] = "".concat(i);
    }
    for (var a in e) {
        !t && !D_.call(e, a) || r && ("length" === a || (0, b_.isIndex)(a, n)) || o.push(a);
    }
    return o;
};

var S_ = {};

Object.defineProperty(S_, "__esModule", {
    value: !0
}), S_.default = function(e) {
    return null == e;
};

var A_ = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(c_, "__esModule", {
    value: !0
});

var O_ = A_(f_), C_ = A_(p_), x_ = A_(S_);

c_.default = function(e) {
    return (0, x_.default)(e) ? [] : (0, O_.default)(e) ? (0, C_.default)(e, void 0) : Object.keys(Object(e));
};

var F_ = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, M_ = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
}, P_ = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(zy, "__esModule", {
    value: !0
}), zy.createAssignFunction = void 0;

var I_ = Hy, k_ = r_, R_ = n_, T_ = P_(c_);

function j_(e) {
    return function(t) {
        for (var r = [], n = 1; n < arguments.length; n++) {
            r[n - 1] = arguments[n];
        }
        var o = -1, i = r.length, a = i > 1 ? r[i - 1] : void 0, s = i > 2 ? r[2] : void 0;
        a = e.length > 3 && "function" == typeof a ? (i--, a) : void 0, s && (0, I_.isIterate)(r[0], r[1], s) && (a = i < 3 ? void 0 : a, 
        i = 1);
        for (var u = Object(t); ++o < i; ) {
            var l = r[o];
            l && e(u, l, o, a);
        }
        return u;
    };
}

zy.createAssignFunction = j_;

var L_ = function(e, t) {
    if ((0, I_.isPrototype)(t) || Array.isArray(t)) {
        (0, k_.copyObject)(t, (0, T_.default)(t), e, void 0);
    } else {
        for (var r in t) {
            Object.hasOwnProperty.call(t, r) && (0, R_.assignValue)(e, r, t[r]);
        }
    }
};

zy.default = function(e) {
    for (var t = [], r = 1; r < arguments.length; r++) {
        t[r - 1] = arguments[r];
    }
    return j_(L_).apply(void 0, M_([ e ], F_(t), !1));
};

var N_ = {};

Object.defineProperty(N_, "__esModule", {
    value: !0
}), N_.default = function(e) {
    for (var t, r = [], n = 1; n < arguments.length; n++) {
        r[n - 1] = arguments[n];
    }
    for (var o = Object.assign({}, e), i = function(e, t) {
        var n = r[e];
        if (Array.isArray(n)) {
            for (var i = 0, a = n.length; i < a; i++) {
                o[i] = n[i];
            }
        } else {
            Object.keys(n).forEach(function(e) {
                o[e] = n[e];
            });
        }
    }, a = 0, s = r.length; a < s; a++) {
        i(a);
    }
    var u = function(e, n) {
        ((null === (t = r[e]) || void 0 === t ? void 0 : t.constructor) ? Object.keys(r[e].constructor.prototype) : []).forEach(function(t) {
            o[t] = r[e].constructor.prototype[t];
        });
    };
    for (a = 0, s = r.length; a < s; a++) {
        u(a);
    }
    return o;
};

var B_ = {};

Object.defineProperty(B_, "__esModule", {
    value: !0
}), B_.default = function(e) {
    return 0 === arguments.length ? [] : Array.isArray(e) ? e : [ e ];
};

var U_ = {}, z_ = {};

Object.defineProperty(z_, "__esModule", {
    value: !0
}), z_.default = function(e) {
    void 0 === e && (e = void 0);
    var t = e;
    return e instanceof Object && (t = e.valueOf()), t != t;
};

var H_ = {}, $_ = {}, G_ = {}, W_ = {};

Object.defineProperty(W_, "__esModule", {
    value: !0
}), W_.default = function(e) {
    return e;
};

var V_ = {}, K_ = {}, q_ = {};

Object.defineProperty(q_, "__esModule", {
    value: !0
});

var J_, X_ = Array.isArray;

function Z_() {
    if (J_) {
        return K_;
    }
    J_ = 1;
    var e = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(K_, "__esModule", {
        value: !0
    }), K_.isKey = void 0;
    var t = e(q_), r = e(dE()), n = /\.|\[(?:[^[\]]*|(["'])(?:(?!\1)[^\\]|\\.)*?\1)\]/, o = /^\w*$/;
    return K_.isKey = function(e, i) {
        if ((0, t.default)(e)) {
            return !1;
        }
        var a = typeof e;
        return !("number" !== a && "symbol" !== a && "boolean" !== a && null != e && !(0, 
        r.default)(e)) || (o.test(e) || !n.test(e) || null != i && e in Object(i));
    }, K_;
}

q_.default = function(e) {
    return X_(e);
};

var Y_, Q_ = {};

function eE() {
    if (Y_) {
        return Q_;
    }
    Y_ = 1;
    var e = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(Q_, "__esModule", {
        value: !0
    }), Q_.toKey = void 0;
    var t = e(dE());
    return Q_.toKey = function(e) {
        if ("string" == typeof e || (0, t.default)(e)) {
            return e;
        }
        var r = "".concat(e);
        return "0" === r && 1 / e == -1 / 0 ? "-0" : r;
    }, Q_;
}

var tE, rE, nE, oE, iE, aE = {}, sE = {};

function uE() {
    if (tE) {
        return sE;
    }
    tE = 1, Object.defineProperty(sE, "__esModule", {
        value: !0
    }), sE.getObjValidPathFromGeneralPath = void 0;
    var e = fE(), t = /(?:\[('|")((?:\\[\s\S]|(?!\1)[^\\])+)\1\]|\[(-?\d+(?:\.\d+)?)\]|\[((?:\\[\s\S]|[^[\]])*?)\]|([\w|!|@|#|$|%|^|&|*|(|)|\-|+|=|{|}|||;|:|<|>|?|,|'|"|！|￥|…|（|）|—|_|、|；|：|《|》|？|，|‘|’|“|”|/|\\]+))/g;
    function r(e) {
        if ("" === e.trim()) {
            return [ e ];
        }
        for (var r, n = []; null !== (r = t.exec(e)); ) {
            r[2] ? n.push("".concat(r[1]).concat(r[2]).concat(r[1])) : r[3] ? n.push(r[3]) : r[4] || "" === r[4] ? n.push("".concat(r[4])) : r[5] && n.push(r[5]);
        }
        return n;
    }
    return sE.default = r, sE.getObjValidPathFromGeneralPath = function(t, n) {
        if ("symbol" == typeof n) {
            return [ n ];
        }
        if (Array.isArray(n)) {
            return n.map(function(t) {
                return (0, e.toStringWithZeroSign)(t);
            });
        }
        var o = (0, e.toStringWithZeroSign)(n);
        return null != t && o in Object(t) ? [ o ] : r(o);
    }, sE;
}

function lE() {
    if (rE) {
        return aE;
    }
    rE = 1;
    var e = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(aE, "__esModule", {
        value: !0
    });
    var t = e(uE()), r = /^\w+$/;
    return aE.default = function(e, n, o) {
        var i = null == e ? void 0 : function(e, n) {
            for (var o, i = 0, a = e, s = ((o = Array.isArray(n) ? n : r.test(n) || n in Object(e) ? [ n ] : (0, 
            t.default)(n)).length); null != a && i < o.length; ) {
                a = a[o[i++]];
            }
            return i && i === s ? a : void 0;
        }(e, n);
        return void 0 === i ? o : i;
    }, aE;
}

function cE() {
    if (nE) {
        return V_;
    }
    nE = 1;
    var e = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(V_, "__esModule", {
        value: !0
    });
    var t = Z_(), r = eE(), n = e(lE());
    return V_.default = function(e) {
        return (0, t.isKey)(e) ? function(t) {
            return null == t ? void 0 : t[(0, r.toKey)(e)];
        } : function(t) {
            return (0, n.default)(t, e);
        };
    }, V_;
}

function fE() {
    if (oE) {
        return G_;
    }
    oE = 1;
    var e = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(G_, "__esModule", {
        value: !0
    }), G_.getRealIterateeWithIdentityDefault = G_.getObjectKeysWithProtoChain = G_.toStringWithZeroSign = G_.falsey = G_.whiteSpace = G_.tagName = void 0;
    var t = e(W_), r = e(cE());
    G_.tagName = function(e) {
        return null === e ? "[object Null]" : void 0 === e ? "[object Undefined]" : Object.prototype.toString.apply(e);
    }, G_.whiteSpace = [ " ", "\t", "\v", "\f", " ", "\ufeff", "\n", "\r", "\u2028", "\u2029", " ", "᠎", " ", " ", " ", " ", " ", " ", " ", " ", " ", " ", " ", " ", " ", "　" ], 
    G_.falsey = [ null, void 0, !1, 0, NaN, "" ];
    return G_.toStringWithZeroSign = function(e) {
        return "symbol" == typeof e ? e : Object.is(-0, e) || e instanceof Number && Object.is(-0, Number(e)) ? "-0" : String(e);
    }, G_.getObjectKeysWithProtoChain = function(e) {
        var t = [];
        if (null == e) {
            return t;
        }
        for (var r in e) {
            r && t.push(r);
        }
        return t;
    }, G_.getRealIterateeWithIdentityDefault = function(e) {
        var n = t.default, o = typeof e;
        return "function" === o ? n = e : "string" === o && (n = (0, r.default)(e)), n;
    }, G_;
}

function dE() {
    if (iE) {
        return $_;
    }
    iE = 1, Object.defineProperty($_, "__esModule", {
        value: !0
    });
    var e = fE();
    return $_.default = function(t) {
        void 0 === t && (t = void 0);
        var r = typeof t;
        return "symbol" === r || "object" === r && null != t && "[object Symbol]" === (0, 
        e.tagName)(t);
    }, $_;
}

var pE = {};

Object.defineProperty(pE, "__esModule", {
    value: !0
}), pE.default = function(e, t) {
    if (null == e) {
        return "";
    }
    if (e && !t) {
        return e.trim();
    }
    var r = new Set(t && t.split(""));
    r.add(" ");
    for (var n = e.split(""), o = 0, i = n.length - 1, a = 0; a < n.length; a++) {
        if (!r.has(n[a])) {
            o = a;
            break;
        }
    }
    for (a = n.length - 1; a >= o; a--) {
        if (!r.has(n[a])) {
            i = a;
            break;
        }
    }
    return n.slice(o, i + 1).join("");
};

var hE = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(H_, "__esModule", {
    value: !0
});

var vE = Hy, gE = hE(dE()), mE = hE(pE);

H_.default = function(e) {
    var t = e;
    if ("number" == typeof t) {
        return t;
    }
    if ((0, gE.default)(t)) {
        return NaN;
    }
    if ((0, vE.isObject)(t)) {
        var r = "function" == typeof t.valueOf ? t.valueOf() : t;
        t = (0, vE.isObject)(r) ? "".concat(r) : r;
    }
    return "string" != typeof t ? 0 === t ? t : +t : function(e) {
        var t = (0, mE.default)(e), r = /^0b[01]+$/i.test(t), n = /^0o[0-7]+$/i.test(t), o = /^[-+]0x[0-9a-f]+$/i.test(t);
        return r || n ? parseInt(t.slice(2), r ? 2 : 8) : o ? NaN : +t;
    }(t);
};

var yE = {}, _E = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(yE, "__esModule", {
    value: !0
}), yE.numMulti = void 0;

var EE = _E(z_), bE = _E(H_);

function wE(e, t) {
    var r = 0;
    try {
        e.toString().indexOf(".") > -1 && (r += e.toString().split(".")[1].length);
    } catch (e) {}
    try {
        t.toString().indexOf(".") > -1 && (r += t.toString().split(".")[1].length);
    } catch (e) {}
    return Number(e.toString().replace(".", "")) * Number(t.toString().replace(".", "")) / Math.pow(10, r);
}

yE.numMulti = wE, yE.default = function(e, t) {
    if (void 0 === t && (t = 0), "number" != typeof Number(e)) {
        return Number.NaN;
    }
    if (e === Number.MAX_SAFE_INTEGER || e === Number.MIN_SAFE_INTEGER) {
        return e;
    }
    var r = (0, EE.default)(t) ? 0 : Math.floor((0, bE.default)(t)), n = Number(e);
    if (0 === r) {
        return Math.floor(n);
    }
    var o = Math.pow(10, Math.abs(r));
    if (o === Number.POSITIVE_INFINITY || o === Number.NEGATIVE_INFINITY) {
        return e;
    }
    if (n >= 0 && 1 / n > 0) {
        if (r > 0) {
            return Math.floor(wE(Math.abs(n), o)) / o;
        }
        if (r < 0) {
            return wE(Math.floor(Math.abs(n) / o), o);
        }
    } else {
        if (r > 0) {
            return -Math.ceil(wE(Math.abs(n), o)) / o;
        }
        if (r < 0) {
            return -wE(Math.ceil(Math.abs(n) / o), o);
        }
    }
    return 0;
};

var DE = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(U_, "__esModule", {
    value: !0
});

var SE = DE(z_), AE = DE(H_), OE = yE;

U_.default = function(e, t) {
    if (void 0 === t && (t = 0), "number" != typeof Number(e)) {
        return Number.NaN;
    }
    if (e === Number.MAX_SAFE_INTEGER || e === Number.MIN_SAFE_INTEGER) {
        return e;
    }
    var r = (0, SE.default)(t) ? 0 : Math.floor((0, AE.default)(t)), n = Number(e);
    if (0 === r) {
        return Math.ceil(n);
    }
    var o = Math.pow(10, Math.abs(r));
    if (o === Number.POSITIVE_INFINITY || o === Number.NEGATIVE_INFINITY) {
        return e;
    }
    if (n >= 0 && 1 / n > 0) {
        if (r > 0) {
            return Math.ceil((0, OE.numMulti)(Math.abs(n), o)) / o;
        }
        if (r < 0) {
            return (0, OE.numMulti)(Math.ceil(Math.abs(n) / o), o);
        }
    } else {
        if (r > 0) {
            return -Math.floor((0, OE.numMulti)(Math.abs(n), o)) / o;
        }
        if (r < 0) {
            return -(0, OE.numMulti)(Math.floor(Math.abs(n) / o), o);
        }
    }
    return 0;
};

var CE = {}, xE = {};

Object.defineProperty(xE, "__esModule", {
    value: !0
}), xE.default = function(e, t, r) {
    var n = e.length, o = t;
    o < 0 && (o = -t > n ? 0 : n + t);
    var i = r > n ? n : r;
    i < 0 && (i += n), n = o > i ? 0 : i - o;
    for (var a = Array(n), s = -1; ++s < n; ) {
        a[s] = e[s + o];
    }
    return a;
};

var FE = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(CE, "__esModule", {
    value: !0
});

var ME = FE(xE);

CE.default = function(e, t) {
    void 0 === t && (t = 1);
    var r = null == e ? 0 : e.length, n = Math.floor, o = Math.ceil, i = n(Number(t));
    if (!i || i < 0) {
        return [];
    }
    for (var a = o(r / i), s = Array(a), u = 0; u < a; u++) {
        s[u] = (0, ME.default)(e, u * i, (u + 1) * i);
    }
    return s;
};

var PE = {};

Object.defineProperty(PE, "__esModule", {
    value: !0
});

var IE = Array.isArray;

function kE(e, t) {
    return e < t ? e : t;
}

function RE(e) {
    return IE(e) ? 0 === e.length ? 0 : 1 === e.length ? Number(e[0]) : NaN : Number(e);
}

PE.default = function(e, t, r) {
    var n = RE(e), o = RE(t), i = RE(r), a = Number.isNaN, s = a(n);
    return o = a(o) ? 0 : o, i = a(i) ? 0 : i, s ? NaN : void 0 === t ? void 0 === r ? n : kE(n, i) : void 0 === r ? kE(n, o) : i < o || n < o ? o : n < i ? n : i;
};

var TE = {}, jE = {}, LE = {};

Object.defineProperty(LE, "__esModule", {
    value: !0
}), LE.default = function(e, t) {
    var r = -1, n = e.length, o = t;
    for (Array.isArray(o) || (o = new Array(n)); ++r < n; ) {
        o[r] = e[r];
    }
    return o;
};

var NE = {}, BE = {}, UE = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, zE = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
};

Object.defineProperty(BE, "__esModule", {
    value: !0
}), BE.getSymbolsIn = BE.getSymbols = void 0;

var HE = Object.prototype.propertyIsEnumerable, $E = Object.getOwnPropertySymbols;

function GE(e) {
    var t = e;
    return null == t ? [] : (t = Object(t), $E(t).filter(function(e) {
        return HE.call(t, e);
    }));
}

BE.getSymbols = GE, BE.getSymbolsIn = function(e) {
    for (var t = e, r = []; t; ) {
        r.push.apply(r, zE([], UE(GE(t)), !1)), t = Object.getPrototypeOf(Object(t));
    }
    return r;
}, Object.defineProperty(NE, "__esModule", {
    value: !0
}), NE.copySymbolsIn = void 0;

var WE = BE, VE = r_;

NE.copySymbolsIn = function(e, t) {
    return (0, VE.copyObject)(e, (0, WE.getSymbolsIn)(e), t, !1);
}, NE.default = function(e, t) {
    return (0, VE.copyObject)(e, (0, WE.getSymbols)(e), t, !1);
};

var KE = {}, qE = {};

Object.defineProperty(qE, "__esModule", {
    value: !0
}), qE.default = function(e) {
    return null !== e && [ "object", "function" ].includes(typeof e);
};

var JE = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(KE, "__esModule", {
    value: !0
});

var XE = JE(p_), ZE = Hy, YE = JE(f_), QE = JE(qE);

function eb(e) {
    if (!(0, QE.default)(e)) {
        return function(e) {
            var t = [];
            if (null == e) {
                return t;
            }
            var r = Object(e);
            for (var n in r) {
                n in r && t.push(n);
            }
            return t;
        }(e);
    }
    var t = (0, ZE.isPrototype)(e), r = [];
    for (var n in e) {
        ("constructor" !== n || !t && Object.prototype.hasOwnProperty.call(e, n)) && r.push(n);
    }
    return r;
}

KE.default = function(e) {
    return (0, YE.default)(e) ? (0, XE.default)(e, !0) : eb(e);
};

var tb = {}, rb = {}, nb = {}, ob = {};

Object.defineProperty(ob, "__esModule", {
    value: !0
}), ob.default = function(e, t) {
    return void 0 === e && (e = void 0), void 0 === t && (t = void 0), e === t || e != e && t != t;
};

var ib = g && g.__values || function(e) {
    var t = "function" == typeof Symbol && Symbol.iterator, r = t && e[t], n = 0;
    if (r) {
        return r.call(e);
    }
    if (e && "number" == typeof e.length) {
        return {
            next: function() {
                return e && n >= e.length && (e = void 0), {
                    value: e && e[n++],
                    done: !e
                };
            }
        };
    }
    throw new TypeError(t ? "Object is not iterable." : "Symbol.iterator is not defined.");
}, ab = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(nb, "__esModule", {
    value: !0
}), nb.assocIndexOf = nb.cacheHas = nb.arrayIncludesWith = nb.arrayIncludes = nb.baseIndexOf = nb.strictIndexOf = void 0;

var sb = Hy, ub = ab(ob);

function lb(e, t, r, n) {
    for (var o = e.length, i = r + (n ? 1 : -1); n ? i-- : ++i < o; ) {
        if (t(e[i], i, e)) {
            return i;
        }
    }
    return -1;
}

function cb(e, t, r) {
    return e.indexOf(t, r);
}

nb.strictIndexOf = cb, nb.baseIndexOf = function(e, t, r) {
    return Number.isNaN(t) ? cb(e, t, r) : lb(e, sb.baseIsNaN, r, !1);
}, nb.arrayIncludes = function(e, t) {
    return e.includes(t);
}, nb.arrayIncludesWith = function(e, t, r) {
    var n, o;
    if (null == e) {
        return !1;
    }
    try {
        for (var i = ib(e), a = i.next(); !a.done; a = i.next()) {
            if (r(t, a.value)) {
                return !0;
            }
        }
    } catch (e) {
        n = {
            error: e
        };
    } finally {
        try {
            a && !a.done && (o = i.return) && o.call(i);
        } finally {
            if (n) {
                throw n.error;
            }
        }
    }
    return !1;
}, nb.cacheHas = function(e, t) {
    return e.has(t);
}, nb.assocIndexOf = function(e, t) {
    for (var r = e.length; r--; ) {
        if ((0, ub.default)(e[r][0], t)) {
            return r;
        }
    }
    return -1;
}, nb.default = lb, Object.defineProperty(rb, "__esModule", {
    value: !0
});

var fb = nb, db = function() {
    function e(e) {
        this.wdkData = [], this.size = 0;
        for (var t = -1, r = null == e ? 0 : e.length; ++t < r; ) {
            var n = e[t];
            this.set(n[0], n[1]);
        }
    }
    return e.prototype.clear = function() {
        this.wdkData = [], this.size = 0;
    }, e.prototype.delete = function(e) {
        var t = this.wdkData, r = (0, fb.assocIndexOf)(t, e);
        return !(r < 0) && (r === t.length - 1 ? t.pop() : t.splice(r, 1), --this.size, 
        !0);
    }, e.prototype.get = function(e) {
        var t = this.wdkData, r = (0, fb.assocIndexOf)(t, e);
        return r < 0 ? void 0 : t[r][1];
    }, e.prototype.has = function(e) {
        return (0, fb.assocIndexOf)(this.wdkData, e) > -1;
    }, e.prototype.set = function(e, t) {
        var r = this.wdkData, n = (0, fb.assocIndexOf)(r, e);
        return n < 0 ? (++this.size, r.push([ e, t ])) : r[n][1] = t, this;
    }, e;
}();

rb.default = db;

var pb = {}, hb = {};

Object.defineProperty(hb, "__esModule", {
    value: !0
});

var vb = "__wdk_hash_undefined__", gb = function() {
    function e(e) {
        this.wdkData = Object.create(null), this.size = 0;
        for (var t = -1, r = null == e ? 0 : e.length; ++t < r; ) {
            var n = e[t];
            this.set(n[0], n[1]);
        }
    }
    return e.prototype.clear = function() {
        this.wdkData = Object.create(null), this.size = 0;
    }, e.prototype.delete = function(e) {
        var t = this.has(e) && delete this.wdkData[e];
        return this.size -= t ? 1 : 0, t;
    }, e.prototype.get = function(e) {
        var t = this.wdkData[e];
        return t === vb ? void 0 : t;
    }, e.prototype.has = function(e) {
        return void 0 !== this.wdkData[e];
    }, e.prototype.set = function(e, t) {
        var r = this.wdkData;
        return this.size += this.has(e) ? 0 : 1, r[e] = void 0 === t ? vb : t, this;
    }, e;
}();

hb.default = gb;

var mb = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(pb, "__esModule", {
    value: !0
});

var yb = mb(hb);

function _b(e, t) {
    var r = e.wdkData;
    return function(e) {
        var t = typeof e;
        return "string" === t || "number" === t || "symbol" === t || "boolean" === t ? "__proto__" !== e : null === e;
    }(t) ? r["string" == typeof t ? "string" : "hash"] : r.map;
}

var Eb = function() {
    function e(e) {
        this.size = 0, this.wdkData = {
            hash: new yb.default(void 0),
            map: new Map,
            string: new yb.default(void 0)
        };
        for (var t = -1, r = null == e ? 0 : e.length; ++t < r; ) {
            var n = e[t];
            this.set(n[0], n[1]);
        }
    }
    return e.prototype.clear = function() {
        this.size = 0, this.wdkData = {
            hash: new yb.default(void 0),
            map: new Map,
            string: new yb.default(void 0)
        };
    }, e.prototype.delete = function(e) {
        var t = _b(this, e).delete(e);
        return this.size -= t ? 1 : 0, t;
    }, e.prototype.get = function(e) {
        return _b(this, e).get(e);
    }, e.prototype.has = function(e) {
        return _b(this, e).has(e);
    }, e.prototype.set = function(e, t) {
        var r = _b(this, e), n = r.size;
        return r.set(e, t), this.size += r.size === n ? 0 : 1, this;
    }, e;
}();

pb.default = Eb;

var bb = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(tb, "__esModule", {
    value: !0
}), tb.Stack = void 0;

var wb = bb(rb), Db = bb(pb), Sb = function() {
    function e(e) {
        this.wdkData = new wb.default(e);
        var t = this.wdkData;
        this.size = t.size;
    }
    return e.prototype.clear = function() {
        this.wdkData = new wb.default(void 0), this.size = 0;
    }, e.prototype.delete = function(e) {
        var t = this.wdkData, r = t.delete(e);
        return this.size = t.size, r;
    }, e.prototype.get = function(e) {
        return this.wdkData.get(e);
    }, e.prototype.has = function(e) {
        return this.wdkData.has(e);
    }, e.prototype.set = function(e, t) {
        var r = this.wdkData;
        if (r instanceof wb.default) {
            var n = r.wdkData;
            if (n.length < 199) {
                return n.push([ e, t ]), this.size = ++r.size, this;
            }
            this.wdkData = new Db.default(n), r = this.wdkData;
        }
        return r.set(e, t), this.size = r.size, this;
    }, e;
}();

tb.Stack = Sb;

var Ab = {}, Ob = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, Cb = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
}, xb = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Ab, "__esModule", {
    value: !0
}), Ab.getAllKeysIn = void 0;

var Fb = xb(c_), Mb = BE;

Ab.getAllKeysIn = function(e) {
    var t = [];
    for (var r in e) {
        Object.hasOwnProperty.call(e, r) && t.push(r);
    }
    return Array.isArray(e) || t.push.apply(t, Cb([], Ob((0, Mb.getSymbolsIn)(e)), !1)), 
    t;
}, Ab.default = function(e) {
    var t = (0, Fb.default)(e);
    return Array.isArray(e) || t.push.apply(t, Cb([], Ob((0, Mb.getSymbols)(e)), !1)), 
    t;
};

var Pb = {};

Object.defineProperty(Pb, "__esModule", {
    value: !0
}), Pb.arrayEach = void 0, Pb.arrayEach = function(e, t) {
    return e.forEach(t), e;
};

var Ib = g && g.__createBinding || (Object.create ? function(e, t, r, n) {
    void 0 === n && (n = r);
    var o = Object.getOwnPropertyDescriptor(t, r);
    o && !("get" in o ? !t.__esModule : o.writable || o.configurable) || (o = {
        enumerable: !0,
        get: function() {
            return t[r];
        }
    }), Object.defineProperty(e, n, o);
} : function(e, t, r, n) {
    void 0 === n && (n = r), e[n] = t[r];
}), kb = g && g.__setModuleDefault || (Object.create ? function(e, t) {
    Object.defineProperty(e, "default", {
        enumerable: !0,
        value: t
    });
} : function(e, t) {
    e.default = t;
}), Rb = g && g.__importStar || function(e) {
    if (e && e.__esModule) {
        return e;
    }
    var t = {};
    if (null != e) {
        for (var r in e) {
            "default" !== r && Object.prototype.hasOwnProperty.call(e, r) && Ib(t, e, r);
        }
    }
    return kb(t, e), t;
}, Tb = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(jE, "__esModule", {
    value: !0
}), jE.initCloneObject = jE.cloneDataView = jE.cloneRegExp = jE.cloneSymbol = jE.cloneTypedArray = jE.cloneArrayBuffer = void 0;

var jb = Hy, Lb = Tb($y), Nb = Tb(LE), Bb = Rb(NE), Ub = r_, zb = Tb(KE), Hb = tb, $b = Tb(h_), Gb = Rb(Ab), Wb = Tb(c_), Vb = Pb, Kb = n_;

function qb(e) {
    var t = new e.constructor(e.byteLength);
    return new Uint8Array(t).set(new Uint8Array(e)), t;
}

function Jb(e, t) {
    var r = t ? qb(e.buffer) : e.buffer;
    return new e.constructor(r, e.byteOffset, e.length);
}

jE.cloneArrayBuffer = qb, jE.cloneTypedArray = Jb;

var Xb = Symbol.prototype.valueOf;

function Zb(e) {
    return Object(Xb.call(e));
}

jE.cloneSymbol = Zb;

var Yb = /\w*$/;

function Qb(e) {
    var t = new e.constructor(e.source, Yb.exec(e));
    return t.lastIndex = e.lastIndex, t;
}

function ew(e, t) {
    var r = t ? qb(e.buffer) : e.buffer;
    return new e.constructor(r, e.byteOffset, e.byteLength);
}

function tw(e) {
    return "function" != typeof e.constructor || (0, jb.isPrototype)(e) ? {} : Object.create(Object.getPrototypeOf(e));
}

jE.cloneRegExp = Qb, jE.cloneDataView = ew, jE.initCloneObject = tw;

var rw = "[object Arguments]", nw = "[object Boolean]", ow = "[object Date]", iw = "[object Map]", aw = "[object Number]", sw = "[object Object]", uw = "[object RegExp]", lw = "[object Set]", cw = "[object String]", fw = "[object Symbol]", dw = "[object ArrayBuffer]", pw = "[object DataView]", hw = "[object Float32Array]", vw = "[object Float64Array]", gw = "[object Int8Array]", mw = "[object Int16Array]", yw = "[object Int32Array]", _w = "[object Uint8Array]", Ew = "[object Uint8ClampedArray]", bw = "[object Uint16Array]", ww = "[object Uint32Array]", Dw = {};

Dw[rw] = !0, Dw["[object Array]"] = !0, Dw[dw] = !0, Dw[pw] = !0, Dw[nw] = !0, Dw[ow] = !0, 
Dw[hw] = !0, Dw[vw] = !0, Dw[gw] = !0, Dw[mw] = !0, Dw[yw] = !0, Dw[iw] = !0, Dw[aw] = !0, 
Dw[sw] = !0, Dw[uw] = !0, Dw[lw] = !0, Dw[cw] = !0, Dw[fw] = !0, Dw[_w] = !0, Dw[Ew] = !0, 
Dw[bw] = !0, Dw[ww] = !0, Dw["[object Error]"] = !1, Dw["[object WeakMap]"] = !1;

var Sw = Object.prototype.hasOwnProperty, Aw = [ hw, vw, gw, mw, yw, _w, Ew, bw, ww ], Ow = [ nw, ow ], Cw = [ aw, cw ];

jE.default = function e(t, r, n, o, i, a) {
    var s, u = 1 & r, l = 2 & r, c = 4 & r;
    if (n && (s = i ? n(t, o, i, a) : n(t)), void 0 !== s) {
        return s;
    }
    if (!(0, jb.isObject)(t)) {
        return t;
    }
    var f = Array.isArray(t), d = (0, Lb.default)(t);
    if (f) {
        if (s = function(e) {
            var t = e.length, r = new e.constructor(t);
            return t && "string" == typeof e[0] && Sw.call(e, "index") && (r.index = e.index, 
            r.input = e.input), r;
        }(t), !u) {
            return (0, Nb.default)(t, s);
        }
    } else {
        var p = "function" == typeof t;
        if (d === sw || d === rw || p && !i) {
            if (s = l || p ? {} : tw(t), !u) {
                return l ? (0, Bb.copySymbolsIn)(t, (0, Ub.copyObject)(t, (0, zb.default)(t), s, !1)) : (0, 
                Bb.default)(t, Object.assign(s, t));
            }
        } else {
            if (p || !Dw[d]) {
                return i ? t : {};
            }
            s = function(e, t, r) {
                var n = e.constructor;
                if (Aw.includes(t)) {
                    return Jb(e, r);
                }
                if (Ow.includes(t)) {
                    return new n(+e);
                }
                if (Cw.includes(t)) {
                    return new n(e);
                }
                switch (t) {
                  case dw:
                    return qb(e);

                  case pw:
                    return ew(e, r);

                  case iw:
                    return new n;

                  case uw:
                    return Qb(e);

                  case lw:
                    return new n;

                  case fw:
                    return Zb(e);

                  default:
                    return;
                }
            }(t, d, u);
        }
    }
    var h = a;
    h || (h = new Hb.Stack(void 0));
    var v, g = h.get(t);
    if (g) {
        return g;
    }
    if (h.set(t, s), d === iw) {
        return t.forEach(function(o, i) {
            s.set(i, e(o, r, n, i, t, h));
        }), s;
    }
    if (d === lw) {
        return t.forEach(function(o) {
            s.add(e(o, r, n, o, t, h));
        }), s;
    }
    if ((0, $b.default)(t)) {
        return s;
    }
    v = c ? l ? Gb.getAllKeysIn : Gb.default : l ? zb.default : Wb.default;
    var m = f ? void 0 : v(t);
    return (0, Vb.arrayEach)(m || t, function(o, i) {
        var a = i, u = o;
        m && (u = t[a = u]), (0, Kb.assignValue)(s, a, e(u, r, n, a, t, h));
    }), s;
};

var xw = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(TE, "__esModule", {
    value: !0
});

var Fw = xw(jE);

TE.default = function(e) {
    return (0, Fw.default)(e, 4, void 0, void 0, void 0, void 0);
};

var Mw = {}, Pw = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Mw, "__esModule", {
    value: !0
});

var Iw = Pw(jE);

Mw.default = function(e) {
    return (0, Iw.default)(e, 5, void 0, void 0, void 0, void 0);
};

var kw = {};

Object.defineProperty(kw, "__esModule", {
    value: !0
}), kw.default = function(e) {
    if (!Array.isArray(e) || null == e) {
        return [];
    }
    for (var t = [], r = 0, n = 0, o = e.length; n < o; n++) {
        e[n] && (t[r++] = e[n]);
    }
    return t;
};

var Rw = {}, Tw = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, jw = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
};

Object.defineProperty(Rw, "__esModule", {
    value: !0
}), Rw.default = function(e) {
    for (var t = [], r = 1; r < arguments.length; r++) {
        t[r - 1] = arguments[r];
    }
    if (0 === arguments.length) {
        return [];
    }
    var n = [];
    Array.isArray(e) ? n.push.apply(n, jw([], Tw(e), !1)) : n.push(e);
    for (var o = 0, i = t.length; o < i; o++) {
        Array.isArray(t[o]) ? n.push.apply(n, jw([], Tw(t[o]), !1)) : n.push(t[o]);
    }
    return n;
};

var Lw = {}, Nw = {}, Bw = {};

Object.defineProperty(Bw, "__esModule", {
    value: !0
});

var Uw = "object" == typeof g && null !== g && g.Object === Object && g;

Bw.default = Uw;

var zw = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Nw, "__esModule", {
    value: !0
});

var Hw = zw(Bw), $w = "object" == typeof globalThis && null !== globalThis && globalThis.Object === Object && globalThis, Gw = "object" == typeof self && null !== self && self.Object === Object && self, Ww = $w || Hw.default || Gw || function() {
    return this;
}();

Nw.default = Ww;

var Vw = g && g.__assign || function() {
    return Vw = Object.assign || function(e) {
        for (var t, r = 1, n = arguments.length; r < n; r++) {
            for (var o in t = arguments[r]) {
                Object.prototype.hasOwnProperty.call(t, o) && (e[o] = t[o]);
            }
        }
        return e;
    }, Vw.apply(this, arguments);
}, Kw = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, qw = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
}, Jw = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Lw, "__esModule", {
    value: !0
});

var Xw = Jw(Nw), Zw = function(e) {
    var t = this;
    this.isLeadingEnabled = !1, this.isTrailingEnabled = !0, this.isMaxWaitEnabled = !1, 
    this.lastInvokeTime = 0, this.debounced = function(e) {
        for (var r = [], n = 1; n < arguments.length; n++) {
            r[n - 1] = arguments[n];
        }
        var o = Date.now(), i = t.shouldInvoke(o);
        if (t.lastArgs = r, t.lastThis = e, t.lastCallTime = o, i) {
            if (void 0 === t.timerId) {
                return t.invokeLeading(t.lastCallTime);
            }
            if (t.isMaxWaitEnabled) {
                return t.cancelTimer(t.timerId), t.timerId = t.startTimer(t.scheduleTimer, t.wait), 
                t.invokeFunc(t.lastCallTime);
            }
        }
        return void 0 === t.timerId && (t.timerId = t.startTimer(t.scheduleTimer, t.wait)), 
        t.debouncedResult;
    }, this.flush = function() {
        return void 0 === t.timerId ? t.debouncedResult : t.invokeTrailing(Date.now());
    }, this.cancel = function() {
        void 0 !== t.timerId && t.cancelTimer(t.timerId), t.lastInvokeTime = 0, t.lastCallTime = void 0, 
        t.lastArgs = void 0, t.lastThis = void 0, t.timerId = void 0;
    }, this.pending = function() {
        return void 0 !== t.timerId;
    }, this.initData = function(e) {
        var r = e.func, n = e.wait, o = e.leading, i = void 0 !== o && o, a = e.trailing, s = void 0 === a || a, u = e.maxWait;
        t.isUsingRAF = void 0 === t.wait && "function" == typeof Xw.default.requestAnimationFrame, 
        t.func = r, t.wait = null != n ? n : 0, t.isMaxWaitEnabled = void 0 !== u, t.maxWait = t.isMaxWaitEnabled ? Math.max(null != u ? u : 0, n) : u, 
        t.isLeadingEnabled = i, t.isTrailingEnabled = s;
    }, this.shouldInvoke = function(e) {
        var r = e - t.lastCallTime, n = e - t.lastInvokeTime;
        return void 0 === t.lastCallTime || r >= t.wait || r < 0 || t.isMaxWaitEnabled && n >= t.maxWait;
    }, this.invokeFunc = function(e) {
        var r = t.lastArgs, n = t.lastThis;
        return t.lastArgs = void 0, t.lastThis = void 0, t.lastInvokeTime = e, t.debouncedResult = t.func.apply(n, r), 
        t.debouncedResult;
    }, this.invokeLeading = function(e) {
        return t.lastInvokeTime = e, t.timerId = t.startTimer(t.scheduleTimer, t.wait), 
        t.isLeadingEnabled ? t.invokeFunc(e) : t.debouncedResult;
    }, this.invokeTrailing = function(e) {
        return t.timerId = void 0, t.isTrailingEnabled && t.lastArgs ? t.invokeFunc(e) : (t.lastArgs = void 0, 
        t.lastThis = void 0, t.debouncedResult);
    }, this.scheduleTimer = function() {
        var e = Date.now();
        t.shouldInvoke(e) ? t.invokeTrailing(e) : t.timerId = t.startTimer(t.scheduleTimer, t.calcRemainingWait(e));
    }, this.startTimer = function(e, r) {
        return t.isUsingRAF ? (Xw.default.cancelAnimationFrame(t.timerId), requestAnimationFrame(e)) : setTimeout(e, r);
    }, this.cancelTimer = function(e) {
        t.isUsingRAF ? Xw.default.cancelAnimationFrame(e) : clearTimeout(e);
    }, this.calcRemainingWait = function(e) {
        var r = e - t.lastCallTime, n = e - t.lastInvokeTime, o = t.wait - r;
        return t.isMaxWaitEnabled ? Math.min(o, t.maxWait - n) : o;
    }, this.initData(e);
};

Lw.default = function(e, t, r) {
    if (void 0 === r && (r = {}), "function" != typeof e) {
        throw new TypeError("Expected a function");
    }
    var n = new Zw(Vw(Vw({}, r), {
        func: e,
        wait: t
    }));
    function o() {
        for (var e = [], t = 0; t < arguments.length; t++) {
            e[t] = arguments[t];
        }
        return n.debounced.apply(n, qw([ this ], Kw(e), !1));
    }
    return o.flush = n.flush, o.cancel = n.cancel, o.pending = n.pending, o;
};

var Yw = {};

Object.defineProperty(Yw, "__esModule", {
    value: !0
}), Yw.default = function(e, t) {
    return null == e || Number.isNaN(e) ? t : e;
};

var Qw = {}, eD = {}, tD = {}, rD = {}, nD = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(rD, "__esModule", {
    value: !0
}), rD.dealWithObject = void 0;

var oD = nD(f_), iD = nD(c_);

var aD = function(e, t, r) {
    for (var n = -1, o = Object(e), i = r(e), a = i.length; a--; ) {
        var s = i[++n];
        if (!1 === t(o[s], s, o)) {
            break;
        }
    }
    return e;
};

var sD, uD, lD = (sD = function(e, t) {
    return e && aD(e, t, iD.default);
}, uD = !1, function(e, t) {
    if (null == e) {
        return e;
    }
    if (!(0, oD.default)(e)) {
        return sD(e, t);
    }
    for (var r = e.length, n = uD ? r : -1, o = Object(e); (uD ? n-- : ++n < r) && !1 !== t(o[n], n, o); ) {}
    return e;
});

rD.dealWithObject = function(e, t) {
    var r = -1, n = (0, oD.default)(e) ? Array(e.length) : [];
    return lD(e, function(e, o, i) {
        n[++r] = t(e, o, i);
    }), n;
};

var cD = {}, fD = {}, dD = {}, pD = {}, hD = {}, vD = {}, gD = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(vD, "__esModule", {
    value: !0
}), vD.SetCache = void 0;

var mD = gD(pb), yD = function() {
    function e(e) {
        this.wdkData = new mD.default(void 0);
        for (var t = -1, r = null == e ? 0 : e.length; ++t < r; ) {
            this.add(e[t]);
        }
    }
    return e.prototype.add = function(e) {
        return this.wdkData.set(e, "__wdk_hash_undefined__"), this;
    }, e.prototype.has = function(e) {
        return this.wdkData.has(e);
    }, e.prototype.push = function(e) {
        this.add(e);
    }, e;
}();

vD.SetCache = yD, Object.defineProperty(hD, "__esModule", {
    value: !0
}), hD.equalArrays = void 0;

var _D = vD, ED = nb;

function bD(e, t, r, n, o, i, a, s) {
    if (e) {
        if (function(e, t, r, n, o, i, a) {
            return !function(e, t) {
                var r = -1, n = null == e ? 0 : e.length;
                for (;++r < n; ) {
                    if (t(e[r], r, e)) {
                        return !0;
                    }
                }
                return !1;
            }(t, function(t, s) {
                if (!(0, ED.cacheHas)(e, s) && (r === t || a(r, t, n, o, i))) {
                    return e.push(s);
                }
            });
        }(e, t, r, n, o, i, a)) {
            return !1;
        }
    } else if (r !== s && !a(r, s, n, o, i)) {
        return !1;
    }
}

hD.equalArrays = function(e, t, r, n, o, i) {
    var a = 1 & r, s = e.length;
    if (!1 === function(e, t, r) {
        if (e !== t && !(r && t > e)) {
            return !1;
        }
    }(s, t.length, a)) {
        return !1;
    }
    var u = function(e, t, r) {
        var n = e.get(t), o = e.get(r);
        if (n && o) {
            return n === r && o === t;
        }
    }(i, e, t);
    if (void 0 !== u) {
        return u;
    }
    var l = -1, c = !0, f = 2 & r ? new _D.SetCache(void 0) : void 0;
    for (i.set(e, t), i.set(t, e); ++l < s; ) {
        var d = void 0, p = e[l], h = t[l];
        if (n && (d = a ? n(h, p, l, t, e, i) : n(p, h, l, e, t, i)), void 0 !== d) {
            if (d) {
                continue;
            }
            c = !1;
            break;
        }
        if (!1 === bD(f, t, p, r, n, i, o, h)) {
            c = !1;
            break;
        }
    }
    return i.delete(e), i.delete(t), c;
};

var wD = {}, DD = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(wD, "__esModule", {
    value: !0
}), wD.equalByTag = void 0;

var SD = DD(ob), AD = hD, OD = Symbol ? Symbol.prototype : void 0, CD = OD ? OD.valueOf : void 0;

function xD(e, t, r) {
    return !(e.byteLength !== t.byteLength || !r(new Uint8Array(e), new Uint8Array(t)));
}

function FD(e, t, r, n, o, i, a) {
    var s = t, u = e, l = 1 & u;
    if (s || (s = PD), r.size !== n.size && !l) {
        return !1;
    }
    var c = o.get(r);
    if (c) {
        return c === n;
    }
    u |= 2, o.set(r, n);
    var f = (0, AD.equalArrays)(s(r), s(n), u, i, a, o);
    return o.delete(r), f;
}

function MD(e) {
    var t = -1, r = Array(e.size);
    return e.forEach(function(e, n) {
        r[++t] = [ n, e ];
    }), r;
}

function PD(e) {
    var t = -1, r = Array(e.size);
    return e.forEach(function(e) {
        r[++t] = e;
    }), r;
}

wD.equalByTag = function(e, t, r, n, o, i, a) {
    var s = e, u = t, l = function(e, t, r, n, o, i, a) {
        var s = e, u = t, l = n, c = function(e) {
            return e;
        };
        return "[object Map]" === r ? FD(l, c = MD, s, u, a, o, i) : "[object Set]" === r ? FD(l, c, s, u, a, o, i) : "[object ArrayBuffer]" === r ? xD(s, u, i) : "[object DataView]" === r ? s.byteLength === u.byteLength && s.byteOffset === u.byteOffset && xD(s = s.buffer, u = u.buffer, i) : void 0;
    }(e, t, r, n, o, i, a);
    if (void 0 !== l) {
        return l;
    }
    switch (r) {
      case "[object Boolean]":
      case "[object Date]":
      case "[object Number]":
        return (0, SD.default)(+s, +u);

      case "[object Error]":
        return function(e, t) {
            return e.name === t.name && e.message === t.message;
        }(s, u);

      case "[object RegExp]":
      case "[object String]":
        return s === "".concat(u);

      case "[object Symbol]":
        if (CD) {
            return CD.call(s) === CD.call(u);
        }
    }
    return !1;
};

var ID = {}, kD = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(ID, "__esModule", {
    value: !0
}), ID.equalObjects = void 0;

var RD = kD(Ab), TD = Object.prototype.hasOwnProperty;

ID.equalObjects = function(e, t, r, n, o, i) {
    var a = 1 & r, s = (0, RD.default)(e), u = s.length;
    if (u !== (0, RD.default)(t).length && !a) {
        return !1;
    }
    for (var l, c = u; c--; ) {
        if (l = s[c], !(a ? l in t : TD.call(t, l))) {
            return !1;
        }
    }
    var f = i.get(e), d = i.get(t);
    if (f && d) {
        return f === t && d === e;
    }
    var p = !0;
    i.set(e, t), i.set(t, e);
    var h = function(e, t, r, n, o, i, a, s, u, l, c, f) {
        for (var d = t, p = n, h = f, v = e; ++d < r; ) {
            var g = i[p = o[d]], m = a[p], y = void 0;
            if (s && (y = e ? s(m, g, p, a, i, u) : s(g, m, p, i, a, u)), !(void 0 === y ? g === m || l(g, m, c, s, u) : y)) {
                h = !1;
                break;
            }
            v || (v = "constructor" === p);
        }
        return {
            skipCtor: v,
            index: d,
            key: p,
            result: h
        };
    }(a, c, u, l, s, e, t, n, i, o, r, p), v = h.skipCtor;
    return p = function(e, t, r, n) {
        var o = e;
        if (o && !t) {
            var i = r.constructor, a = n.constructor;
            i === a || !("constructor" in r) || !("constructor" in n) || "function" == typeof i && i instanceof i && "function" == typeof a && a instanceof a || (o = !1);
        }
        return o;
    }(p = h.result, v, e, t), i.delete(e), i.delete(t), p;
};

var jD = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(pD, "__esModule", {
    value: !0
}), pD.baseIsEqual = void 0;

var LD = jD($y), ND = tb, BD = hD, UD = wD, zD = ID, HD = jD(v_), $D = jD(q_), GD = jD(h_);

pD.baseIsEqual = function e(t, r, n, o, i) {
    return t === r || (null == t || null == r || !(0, HD.default)(t) && !(0, HD.default)(r) ? Number.isNaN(t) && Number.isNaN(r) : function(e, t, r, n, o, i) {
        var a = i, s = (0, $D.default)(e), u = (0, $D.default)(t), l = s ? KD : (0, LD.default)(e), c = u ? KD : (0, 
        LD.default)(t), f = (l = l === VD ? qD : l) === qD, d = (c = c === VD ? qD : c) === qD, p = l === c, h = function(e, t, r, n, o, i, a, s, u, l) {
            var c = r;
            if (e && !t) {
                return c || (c = new ND.Stack(void 0)), n || (0, GD.default)(o) ? (0, BD.equalArrays)(o, i, a, s, u, c) : (0, 
                UD.equalByTag)(o, i, l, a, s, u, c);
            }
            return NaN;
        }(p, f, a, s, e, t, r, n, o, l);
        if (!Number.isNaN(h)) {
            return h;
        }
        var v = function(e, t, r, n, o, i, a, s) {
            var u = i;
            if (!(e & WD)) {
                var l = t && JD.call(r, "__wrapped__"), c = o && JD.call(n, "__wrapped__");
                if (l || c) {
                    var f = l ? r.value() : r, d = c ? n.value() : n;
                    return u || (u = new ND.Stack(void 0)), a(f, d, e, s, u);
                }
            }
            return NaN;
        }(r, f, e, t, d, a, o, n);
        if (!Number.isNaN(v)) {
            return v;
        }
        if (!p) {
            return !1;
        }
        a || (a = new ND.Stack(void 0));
        return (0, zD.equalObjects)(e, t, r, n, o, a);
    }(t, r, n, o, e, i));
};

var WD = 1, VD = "[object Arguments]", KD = "[object Array]", qD = "[object Object]", JD = Object.prototype.hasOwnProperty;

Object.defineProperty(dD, "__esModule", {
    value: !0
}), dD.baseIsMatch = void 0;

var XD = tb, ZD = pD;

function YD(e, t, r, n, o, i, a, s) {
    if (e && t[2]) {
        if (void 0 === r && !(n in o)) {
            return !1;
        }
    } else {
        var u = new XD.Stack(void 0), l = void 0;
        if (i && (l = i(r, a, n, o, s, u)), !(void 0 === l ? (0, ZD.baseIsEqual)(a, r, 3, i, u) : l)) {
            return !1;
        }
    }
}

dD.baseIsMatch = function(e, t, r, n) {
    var o, i = e, a = r.length, s = a, u = !n;
    if (null == i) {
        return !s;
    }
    for (i = Object(i); a--; ) {
        if (o = r[a], u && o[2] ? o[1] !== i[o[0]] : !(o[0] in i)) {
            return !1;
        }
    }
    for (;++a < s; ) {
        var l = (o = r[a])[0];
        if (!1 === YD(u, o, i[l], l, i, n, o[1], t)) {
            return !1;
        }
    }
    return !0;
};

var QD = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(fD, "__esModule", {
    value: !0
}), fD.baseMatches = void 0;

var eS = dD, tS = QD(qE), rS = QD(c_);

function nS(e) {
    return !Number.isNaN(e) && !(0, tS.default)(e);
}

fD.baseMatches = function(e) {
    var t = function(e) {
        var t = (0, rS.default)(e), r = t.length;
        for (;r--; ) {
            var n = t[r], o = e[n];
            t[r] = [ n, o, nS(o) ];
        }
        return t;
    }(e);
    return 1 === t.length && t[0][2] ? function(e, t) {
        return function(r) {
            return null != r && (r[e] === t && (void 0 !== t || e in Object(r)));
        };
    }(t[0][0], t[0][1]) : function(r) {
        return r === e || (0, eS.baseIsMatch)(r, e, t, void 0);
    };
};

var oS = {}, iS = {}, aS = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(iS, "__esModule", {
    value: !0
}), iS.getDeepProperties = void 0;

var sS = eE(), uS = aS(q_), lS = aS(dE());

iS.getDeepProperties = function(e) {
    return function(t) {
        return function(e, t) {
            var r = function(e, t) {
                if ((0, uS.default)(e)) {
                    return e;
                }
                if ((0, lS.default)(e) || e in t) {
                    return [ e ];
                }
                return function(e) {
                    var t = [];
                    "." === e[0] && t.push("");
                    return e.replace(cS, function(e, r, n, o) {
                        return t.push(n ? o.replace(fS, "$1") : r || e), e;
                    }), t;
                }(function(e) {
                    if ((0, lS.default)(e)) {
                        return e;
                    }
                    return "".concat(e);
                }(e));
            }(t, e), n = e, o = 0, i = r.length;
            for (;null != n && o < i; ) {
                n = n[(0, sS.toKey)(r[o++])];
            }
            return o && o === i ? n : void 0;
        }(t, e);
    };
};

var cS = /[^.[\]]+|\[(?:(-?\d+(?:\.\d+)?)|(["'])((?:(?!\2)[^\\]|\\.)*?)\2)\]|(?=(?:\.|\[\])(?:\.|\[\]|$))/g, fS = /\\(\\)?/g;

Object.defineProperty(oS, "__esModule", {
    value: !0
}), oS.getProperties = void 0;

var dS = Z_(), pS = eE(), hS = iS;

oS.getProperties = function(e) {
    return (0, dS.isKey)(e) ? function(e) {
        return function(t) {
            return null == t ? void 0 : t[e];
        };
    }((0, pS.toKey)(e)) : (0, hS.getDeepProperties)(e);
}, Object.defineProperty(cD, "__esModule", {
    value: !0
}), cD.baseIteratee = void 0;

var vS = fD, gS = oS;

function mS(e) {
    return e;
}

cD.baseIteratee = function(e) {
    return "function" == typeof e ? e : null == e ? mS : "object" == typeof e ? (0, 
    vS.baseMatches)(e) : (0, gS.getProperties)(e);
}, Object.defineProperty(tD, "__esModule", {
    value: !0
});

var yS = rD, _S = cD;

function ES(e, t) {
    return e.map(t);
}

tD.default = function(e, t) {
    return (Array.isArray(e) ? ES : yS.dealWithObject)(e, (0, _S.baseIteratee)(t));
};

var bS = g && g.__values || function(e) {
    var t = "function" == typeof Symbol && Symbol.iterator, r = t && e[t], n = 0;
    if (r) {
        return r.call(e);
    }
    if (e && "number" == typeof e.length) {
        return {
            next: function() {
                return e && n >= e.length && (e = void 0), {
                    value: e && e[n++],
                    done: !e
                };
            }
        };
    }
    throw new TypeError(t ? "Object is not iterable." : "Symbol.iterator is not defined.");
}, wS = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(eD, "__esModule", {
    value: !0
}), eD.baseDifference = void 0;

var DS = nb, SS = wS(tD), AS = vD;

eD.baseDifference = function(e, t, r, n) {
    var o, i, a = DS.arrayIncludes, s = !0, u = [], l = t, c = l.length, f = "function" == typeof r;
    if (!(null == e ? void 0 : e.length)) {
        return u;
    }
    r && (l = (0, SS.default)(l, function(e) {
        return f ? r(e) : e[r];
    })), n ? (a = DS.arrayIncludesWith, s = !1) : l.length >= 200 && (a = DS.cacheHas, 
    s = !1, l = new AS.SetCache(l));
    var d = !1;
    try {
        for (var p = bS(e), h = p.next(); !h.done; h = p.next()) {
            var v = h.value, g = v;
            if (r && (g = f ? r(v) : v[r]), v = n || 0 !== v ? v : 0, s && !Number.isNaN(g)) {
                for (var m = c; m--; ) {
                    if (l[m] === g) {
                        d = !0;
                        break;
                    }
                }
                if (d) {
                    d = !1;
                    continue;
                }
                u.push(v);
            } else {
                a(l, g, n) || u.push(v);
            }
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            h && !h.done && (i = p.return) && i.call(p);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return u;
};

var OS = {}, CS = g && g.__values || function(e) {
    var t = "function" == typeof Symbol && Symbol.iterator, r = t && e[t], n = 0;
    if (r) {
        return r.call(e);
    }
    if (e && "number" == typeof e.length) {
        return {
            next: function() {
                return e && n >= e.length && (e = void 0), {
                    value: e && e[n++],
                    done: !e
                };
            }
        };
    }
    throw new TypeError(t ? "Object is not iterable." : "Symbol.iterator is not defined.");
}, xS = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, FS = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
};

Object.defineProperty(OS, "__esModule", {
    value: !0
}), OS.baseFlatten = void 0;

var MS = Hy;

OS.baseFlatten = function e(t, r, n, o, i) {
    var a, s, u = n;
    u || (u = MS.isFlattenable);
    var l = i;
    if (l || (l = []), null == t) {
        return l;
    }
    try {
        for (var c = CS(t), f = c.next(); !f.done; f = c.next()) {
            var d = f.value;
            r > 0 && u(d) ? r > 1 ? e(d, r - 1, u, o, l) : l.push.apply(l, FS([], xS(d), !1)) : o || (l[l.length] = d);
        }
    } catch (e) {
        a = {
            error: e
        };
    } finally {
        try {
            f && !f.done && (s = c.return) && s.call(c);
        } finally {
            if (a) {
                throw a.error;
            }
        }
    }
    return l;
};

var PS = {}, IS = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(PS, "__esModule", {
    value: !0
});

var kS = IS(f_), RS = IS(v_);

PS.default = function(e) {
    return (0, RS.default)(e) && (0, kS.default)(e);
};

var TS = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Qw, "__esModule", {
    value: !0
});

var jS = eD, LS = OS, NS = TS(PS);

Qw.default = function(e) {
    for (var t = [], r = 1; r < arguments.length; r++) {
        t[r - 1] = arguments[r];
    }
    return (0, NS.default)(e) ? (0, jS.baseDifference)(e, (0, LS.baseFlatten)(t, 1, NS.default, !0, void 0), void 0, void 0) : [];
};

var BS = {};

Object.defineProperty(BS, "__esModule", {
    value: !0
}), BS.default = function(e, t) {
    return void 0 === e && void 0 !== t ? Number(t) : void 0 !== e && void 0 === t ? Number(e) : e === t && void 0 === t ? 1 : Number(e) / Number(t);
};

var US = {}, zS = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(US, "__esModule", {
    value: !0
});

var HS = zS(yE);

US.default = function(e, t) {
    if (void 0 === t && (t = 1), !Array.isArray(e) || t >= e.length) {
        return [];
    }
    if (t < 0) {
        return e;
    }
    var r = 0, n = (0, HS.default)(t);
    n >>>= 0;
    for (var o = e.length - n >>> 0, i = Array(o); r < o; ) {
        i[r] = e[r + n], r += 1;
    }
    return i;
};

var $S = {}, GS = {}, WS = {}, VS = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(WS, "__esModule", {
    value: !0
});

var KS = VS(H_), qS = VS(z_), JS = 1 / 0, XS = Number.MAX_VALUE;

WS.default = function(e) {
    if (!e) {
        return 0 === e ? e : 0;
    }
    var t = (0, KS.default)(e);
    return t === JS || t === -1 / 0 ? (t < 0 ? -1 : 1) * XS : (0, qS.default)(t) ? 0 : t;
};

var ZS = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(GS, "__esModule", {
    value: !0
});

var YS = ZS(WS);

GS.default = function(e) {
    var t = (0, YS.default)(e), r = t % 1;
    return r ? t - r : t;
};

var QS = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty($S, "__esModule", {
    value: !0
});

var eA = Hy, tA = QS(GS);

$S.default = function(e, t) {
    return void 0 === t && (t = 1), !(0, eA.isArray)(e) || t >= e.length ? [] : t <= 0 ? e : e.slice(0, e.length - (0, 
    tA.default)(t));
};

var rA = {};

Object.defineProperty(rA, "__esModule", {
    value: !0
}), rA.default = function(e, t, r) {
    var n = e.length, o = r;
    (o = void 0 === o ? n : +o) < 0 || Number.isNaN(o) ? o = 0 : o > n && (o = n);
    var i = o;
    return (o -= t.length) >= 0 && e.slice(o, i) === t;
};

var nA = {}, oA = {}, iA = g && g.__values || function(e) {
    var t = "function" == typeof Symbol && Symbol.iterator, r = t && e[t], n = 0;
    if (r) {
        return r.call(e);
    }
    if (e && "number" == typeof e.length) {
        return {
            next: function() {
                return e && n >= e.length && (e = void 0), {
                    value: e && e[n++],
                    done: !e
                };
            }
        };
    }
    throw new TypeError(t ? "Object is not iterable." : "Symbol.iterator is not defined.");
}, aA = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(oA, "__esModule", {
    value: !0
});

var sA = aA(f_);

oA.default = function(e, t) {
    var r, n, o, i, a = Object(e);
    if (Array.isArray(a)) {
        for (var s = -1, u = e.length; ++s < u && !1 !== t(e[s], s); ) {}
    } else if ((0, sA.default)(a)) {
        try {
            for (var l = iA(a), c = l.next(); !c.done; c = l.next()) {
                if (!1 === t(c.value)) {
                    break;
                }
            }
        } catch (e) {
            r = {
                error: e
            };
        } finally {
            try {
                c && !c.done && (n = l.return) && n.call(l);
            } finally {
                if (r) {
                    throw r.error;
                }
            }
        }
    } else {
        var f = Object.keys(a);
        try {
            for (var d = iA(f), p = d.next(); !p.done; p = d.next()) {
                var h = p.value;
                if (!1 === t(a[h], h)) {
                    break;
                }
            }
        } catch (e) {
            o = {
                error: e
            };
        } finally {
            try {
                p && !p.done && (i = d.return) && i.call(d);
            } finally {
                if (o) {
                    throw o.error;
                }
            }
        }
    }
    return e;
};

var uA = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(nA, "__esModule", {
    value: !0
});

var lA = uA(oA).default;

nA.default = lA;

var cA = {}, fA = {}, dA = {};

!function(e) {
    var t;
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.ObjType = e.getType = void 0, e.getType = function(e) {
        return Object.prototype.toString.call(e);
    }, (t = e.ObjType || (e.ObjType = {})).Arguments = "[object Arguments]", t.Array = "[object Array]", 
    t.AsyncFunction = "[object AsyncFunction]", t.Boolean = "[object Boolean]", t.Date = "[object Date]", 
    t.DOMException = "[object DOMException]", t.Error = "[object Error]", t.Function = "[object Function]", 
    t.GeneratorFunction = "[object GeneratorFunction]", t.Map = "[object Map]", t.Number = "[object Number]", 
    t.Null = "[object Null]", t.Object = "[object Object]", t.Promise = "[object Promise]", 
    t.Proxy = "[object Proxy]", t.RegExp = "[object RegExp]", t.Set = "[object Set]", 
    t.String = "[object String]", t.Symbol = "[object Symbol]", t.Undefined = "[object Undefined]", 
    t.WeakMap = "[object WeakMap]", t.WeakSet = "[object WeakSet]", t.ArrayBuffer = "[object ArrayBuffer]", 
    t.DataView = "[object DataView]", t.Float32Array = "[object Float32Array]", t.Float64Array = "[object Float64Array]", 
    t.Int8Array = "[object Int8Array]", t.Int16Array = "[object Int16Array]", t.Int32Array = "[object Int32Array]", 
    t.Uint8Array = "[object Uint8Array]", t.Uint8ClampedArray = "[object Uint8ClampedArray]", 
    t.Uint16Array = "[object Uint16Array]", t.Uint32Array = "[object Uint32Array]";
}(dA), function(e) {
    var t = g && g.__values || function(e) {
        var t = "function" == typeof Symbol && Symbol.iterator, r = t && e[t], n = 0;
        if (r) {
            return r.call(e);
        }
        if (e && "number" == typeof e.length) {
            return {
                next: function() {
                    return e && n >= e.length && (e = void 0), {
                        value: e && e[n++],
                        done: !e
                    };
                }
            };
        }
        throw new TypeError(t ? "Object is not iterable." : "Symbol.iterator is not defined.");
    };
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.basicCompareArray = e.basicCompareMap = e.basicCompareSet = e.basicCompareObject = e.basicCompare = e.wrapIteratee = e.warpIterateeFromKey = e.wrapIterateeFromObject = e.getValidIndex = e.checkArrayLenValid = e.ArrayDirection = e.CompareModel = void 0;
    var r, n, o = dA;
    function i(e) {
        return function(t) {
            return s(t, e);
        };
    }
    function a(e) {
        return function(t) {
            if (null != t) {
                return e && e in t ? t[e] : void 0;
            }
        };
    }
    function s(e, t, n) {
        if (void 0 === n && (n = r.INCLUDE), "object" != typeof e) {
            return e === t;
        }
        var i = (0, o.getType)(e);
        return i === (0, o.getType)(t) && (i === o.ObjType.Object ? u(e, t, n) : i === o.ObjType.Array ? f(e, t, n) : i === o.ObjType.Map ? c(e, t, n) : i === o.ObjType.Set ? l(e, t, n) : i === o.ObjType.Error ? e.name === t.name && e.message === t.message : e === t);
    }
    function u(e, t, n) {
        for (var o in void 0 === n && (n = r.INCLUDE), t) {
            if (!s(e[o], t[o], n)) {
                return !1;
            }
        }
        return !0;
    }
    function l(e, n, o) {
        var i, a, u, l;
        void 0 === o && (o = r.INCLUDE);
        var c = e.size, f = n.size;
        if (0 === c && 0 === f) {
            return !0;
        }
        if (f > c) {
            return !1;
        }
        try {
            for (var d = t(n), p = d.next(); !p.done; p = d.next()) {
                var h = p.value;
                try {
                    for (var v = (u = void 0, t(e)), g = v.next(); !g.done; g = v.next()) {
                        if (!s(g.value, h, o)) {
                            return !1;
                        }
                    }
                } catch (e) {
                    u = {
                        error: e
                    };
                } finally {
                    try {
                        g && !g.done && (l = v.return) && l.call(v);
                    } finally {
                        if (u) {
                            throw u.error;
                        }
                    }
                }
            }
        } catch (e) {
            i = {
                error: e
            };
        } finally {
            try {
                p && !p.done && (a = d.return) && a.call(d);
            } finally {
                if (i) {
                    throw i.error;
                }
            }
        }
        return !0;
    }
    function c(e, n, o) {
        var i, a;
        void 0 === o && (o = r.INCLUDE);
        var u = e.size, l = n.size;
        if (0 === u && 0 === l) {
            return !0;
        }
        if (l > u) {
            return !1;
        }
        try {
            for (var c = t(n.keys()), f = c.next(); !f.done; f = c.next()) {
                var d = f.value;
                if (!e.has(d) || !s(e.get(d), n.get(d), o)) {
                    return !1;
                }
            }
        } catch (e) {
            i = {
                error: e
            };
        } finally {
            try {
                f && !f.done && (a = c.return) && a.call(c);
            } finally {
                if (i) {
                    throw i.error;
                }
            }
        }
        return !0;
    }
    function f(e, t, n) {
        void 0 === n && (n = r.INCLUDE);
        var o = e.length, i = t.length;
        if (0 === o && 0 === i) {
            return !0;
        }
        if (n !== r.INCLUDE) {
            return o === i;
        }
        if (i > o) {
            return !1;
        }
        for (var a = 0; a < o; a++) {
            if (s(e[a], t[0])) {
                for (var u = !0, l = 1; l < i; l++) {
                    if (!s(e[a + l], t[l])) {
                        return u = !1, !1;
                    }
                }
                if (u) {
                    return !0;
                }
            }
        }
        return !1;
    }
    !function(e) {
        e.EQUAL = "equal", e.INCLUDE = "include";
    }(r = e.CompareModel || (e.CompareModel = {})), (n = e.ArrayDirection || (e.ArrayDirection = {})).LEFT = "left", 
    n.RIGHT = "right", e.checkArrayLenValid = function(e) {
        return null == e || (!e.length || 0 === e.length);
    }, e.getValidIndex = function(e, t) {
        if (void 0 === e && (e = 0), void 0 === t && (t = 0), null == e) {
            return t;
        }
        if (e === 1 / 0 || e === -1 / 0) {
            return e > 0 ? Number.MAX_SAFE_INTEGER : Number.MIN_SAFE_INTEGER;
        }
        var r = Number.isInteger(e) ? e : Number.parseInt(e, 10);
        return Number.isNaN(r) ? t : r;
    }, e.wrapIterateeFromObject = i, e.warpIterateeFromKey = a, e.wrapIteratee = function(e) {
        var t;
        return "function" == typeof e ? e : "object" == typeof e ? i(Array.isArray(e) ? ((t = {})[e[0]] = e[1], 
        t) : e) : a(e);
    }, e.basicCompare = s, e.basicCompareObject = u, e.basicCompareSet = l, e.basicCompareMap = c, 
    e.basicCompareArray = f;
}(fA);

var pA = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(cA, "__esModule", {
    value: !0
});

var hA = fA, vA = pA(f_), gA = pA(W_);

cA.default = function(e, t) {
    void 0 === t && (t = gA.default);
    var r = Object(e), n = (0, vA.default)(r), o = [], i = (0, hA.wrapIteratee)(t);
    if (n) {
        for (s = 0; s < r.length; s++) {
            i(u = r[s], s, r) && o.push(u);
        }
    } else {
        for (var a = Object.keys(r), s = 0; s < a.length; s++) {
            var u;
            i(u = r[a[s]], s, r) && o.push(u);
        }
    }
    return o;
};

var mA = {}, yA = {};

Object.defineProperty(yA, "__esModule", {
    value: !0
});

var _A = fA;

yA.default = function(e, t, r) {
    if ((0, _A.checkArrayLenValid)(e)) {
        return -1;
    }
    var n = (0, _A.getValidIndex)(r, 0);
    return function(e, t, r) {
        if (void 0 === r && (r = 0), 0 === r) {
            return e.findIndex(t);
        }
        for (var n = r, o = e.length; n < o; n++) {
            if (t(e[n], n, e)) {
                return n;
            }
        }
        return -1;
    }(e, (0, _A.wrapIteratee)(t), n >= 0 ? n : Math.max(n + e.length, 0));
};

var EA = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(mA, "__esModule", {
    value: !0
});

var bA = fA, wA = EA(f_), DA = EA(yA);

mA.default = function(e, t, r) {
    var n = Object(e), o = -1;
    if ((0, wA.default)(n)) {
        if ((o = (0, DA.default)(e, t, r)) > -1) {
            return n[o];
        }
    } else {
        var i = (0, bA.wrapIteratee)(t), a = Object.keys(n);
        if (o = (0, DA.default)(a, function(e) {
            return i(n[e], e, n);
        }, r), o > -1) {
            return n[a[o]];
        }
    }
};

var SA = {}, AA = {};

!function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.getStartIndex = void 0;
    e.getStartIndex = function(e, t) {
        var r = void 0 !== t ? Number(t) : e.length;
        return r = [ null, !1, 0, NaN, "" ].includes(t) ? 0 : r, Math.ceil(r < 0 ? Math.max(e.length + r, 0) : r);
    }, e.default = function(t, r, n) {
        if (!Array.isArray(t)) {
            return -1;
        }
        for (var o = -1, i = (0, e.getStartIndex)(t, n), a = i > t.length - 1 ? t.length - 1 : i; a >= 0; a--) {
            if (t[a] === r) {
                o = a;
                break;
            }
        }
        return o;
    };
}(AA), Object.defineProperty(SA, "__esModule", {
    value: !0
});

var OA = AA, CA = fA;

SA.default = function(e, t, r) {
    if (!Array.isArray(e)) {
        return -1;
    }
    for (var n = (0, OA.getStartIndex)(e, r), o = (0, CA.wrapIteratee)(t), i = n > e.length - 1 ? e.length - 1 : n; i >= 0; i--) {
        if (o(e[i], i, e)) {
            return i;
        }
    }
    return -1;
};

var xA = {}, FA = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(xA, "__esModule", {
    value: !0
});

var MA = FA(f_), PA = FA(q_), IA = Hy;

function kA(e, t) {
    for (var r = e.length, n = 0, o = t.length; n < o; n++) {
        e[r + n] = t[n];
    }
    return e;
}

xA.default = function(e) {
    if (!(0, MA.default)(e)) {
        return [];
    }
    var t = [];
    if ((0, PA.default)(e)) {
        for (var r = 0, n = e.length; r < n; r++) {
            (0, IA.isFlattenable)(e[r]) ? kA(t, e[r]) : t.push(e[r]);
        }
        return t;
    }
    for (r = 0, n = e.length; r < n; r++) {
        t.push(e[r]);
    }
    return t;
};

var RA = {}, TA = {};

Object.defineProperty(TA, "__esModule", {
    value: !0
});

var jA = fE();

TA.default = function(e) {
    return "string" == typeof e || "object" == typeof e && "[object String]" === (0, 
    jA.tagName)(e);
};

var LA = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(RA, "__esModule", {
    value: !0
});

var NA = Hy, BA = LA(f_), UA = LA(TA);

function zA(e, t) {
    for (var r = 0, n = e.length; r < n; r++) {
        var o = e[r];
        (0, BA.default)(o) && !(0, UA.default)(o) ? zA(o, t) : t.push(o);
    }
}

RA.default = function(e) {
    if (!(0, BA.default)(e)) {
        return [];
    }
    for (var t = [], r = 0, n = e.length; r < n; r++) {
        (0, NA.isFlattenable)(e[r]) ? zA(e[r], t) : t.push(e[r]);
    }
    return t;
};

var HA = {};

Object.defineProperty(HA, "__esModule", {
    value: !0
}), HA.default = function(e, t) {
    if (Array.isArray(e)) {
        for (var r = -1, n = e.length; ++r < n; ) {
            t(e[r], r);
        }
    } else {
        for (var o in e) {
            t(e[o], o);
        }
    }
    return e;
};

var $A = {}, GA = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty($A, "__esModule", {
    value: !0
});

var WA = GA(qE), VA = GA(S_), KA = function(e, t, r) {
    Object.prototype.hasOwnProperty.call(e, t) || (e[t] = []), e[t].push(r);
};

$A.default = function(e, t) {
    return (0, VA.default)(e) ? {} : "[object Array]" === Object.prototype.toString.call(e) ? function(e, t) {
        var r = {};
        return (0, VA.default)(t) ? e.forEach(function(e) {
            KA(r, "".concat(e), e);
        }) : "string" == typeof t ? e.forEach(function(e) {
            var n = e[t];
            KA(r, "".concat(n), e);
        }) : e.forEach(function(e) {
            if ((Array.isArray(e) || (0, WA.default)(e)) && "number" == typeof t) {
                KA(r, e[t], e);
            } else {
                var n = t && t(e);
                KA(r, "".concat(n), e);
            }
        }), r;
    }(e, t) : function(e, t) {
        var r = {};
        return Object.keys(e).forEach(function(n) {
            var o = t && t(e[n]);
            KA(r, "".concat(o), e[n]);
        }), r;
    }(e, t);
};

var qA = {}, JA = {}, XA = {};

function ZA(e, t) {
    if ("function" != typeof e || null != t && "function" != typeof t) {
        throw new TypeError("Expected a function");
    }
    function r() {
        for (var n = [], o = 0; o < arguments.length; o++) {
            n[o] = arguments[o];
        }
        var i = t ? t.apply(this, n) : n[0], a = r.cache;
        if (a.has(i)) {
            return a.get(i);
        }
        var s = e.apply(this, n);
        return r.cache = a.set(i, s) || a, s;
    }
    return r.cache = new (ZA.Cache || Map), r;
}

Object.defineProperty(XA, "__esModule", {
    value: !0
}), XA.memoizeCapped = void 0, ZA.Cache = Map;

XA.memoizeCapped = function(e) {
    var t, r = ZA(e, function(e) {
        return 500 === t.size && t.clear(), e;
    });
    return t = r.cache, r;
}, XA.default = ZA;

var YA = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(JA, "__esModule", {
    value: !0
});

var QA = Hy, eO = Z_(), tO = YA(q_), rO = YA(dE());

function nO(e) {
    return (0, rO.default)(e) ? e : "".concat(e);
}

var oO = /[^.[\]]+|\[(?:(-?\d+(?:\.\d+)?)|(["'])((?:(?!\2)[^\\]|\\.)*?)\2)\]|(?=(?:\.|\[\])(?:\.|\[\]|$))/g, iO = /\\(\\)?/g, aO = (0, 
XA.memoizeCapped)(function(e) {
    var t = [];
    return "." === e[0] && t.push(""), e.replace(oO, function(e, r, n, o) {
        return t.push(n ? o.replace(iO, "$1") : r || e), e;
    }), t;
});

function sO(e, t) {
    if (!(0, tO.default)(e) && !(0, QA.isArguments)(e)) {
        return !1;
    }
    var r = Number(t);
    return r > -1 && r < e.length;
}

JA.default = function(e, t, r) {
    for (var n = function(e, t) {
        return (0, tO.default)(e) ? e : (0, eO.isKey)(e, t) ? [ e ] : aO(nO(e));
    }(t, e), o = e, i = 0, a = n.length; i < a; i++) {
        var s = nO(n[i]);
        if (!r(o, s) && !sO(o, s)) {
            return !1;
        }
        o = o[s];
    }
    return !0;
};

var uO = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(qA, "__esModule", {
    value: !0
});

var lO = uO(JA), cO = Object.prototype.hasOwnProperty;

function fO(e, t) {
    return null != e && cO.call(e, t);
}

qA.default = function(e, t) {
    return null != e && (0, lO.default)(e, t, fO);
};

var dO = {};

Object.defineProperty(dO, "__esModule", {
    value: !0
}), dO.default = function(e) {
    if (Array.isArray(e) && 0 !== e.length) {
        return e[0];
    }
};

var pO = {}, hO = {}, vO = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(hO, "__esModule", {
    value: !0
});

var gO = vO(c_);

hO.default = function(e) {
    return null == e ? [] : (0, gO.default)(e).map(function(t) {
        return e[t];
    });
};

var mO = {};

Object.defineProperty(mO, "__esModule", {
    value: !0
}), mO.default = function(e, t, r) {
    if (null == e) {
        return -1;
    }
    for (var n = Number.isNaN(Number(r)) ? 0 : Number(r), o = n = Math.round(Number(n) < 0 ? Math.max(e.length + Number(n), 0) : n), i = e.length; o < i; o++) {
        if (e[o] === t || Number.isNaN(t) && Number.isNaN(e[o])) {
            return o;
        }
    }
    return -1;
};

var yO = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(pO, "__esModule", {
    value: !0
});

var _O = yO(f_), EO = yO(hO), bO = yO(H_), wO = yO(TA), DO = yO(mO);

pO.default = function(e, t, r) {
    if (void 0 === r && (r = 0), null == t) {
        return !1;
    }
    var n = (0, _O.default)(e) ? e : (0, EO.default)(e), o = r ? (0, bO.default)(r) : 0;
    return o < 0 && (o = Math.max(n.length + o, 0)), (0, wO.default)(n) ? o <= n.length && n.indexOf(t, o) > -1 : n.length && (0, 
    DO.default)(n, t, o) > -1;
};

var SO = {}, AO = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, OO = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
};

Object.defineProperty(SO, "__esModule", {
    value: !0
});

var CO = Hy;

SO.default = function() {
    for (var e = [], t = 0; t < arguments.length; t++) {
        e[t] = arguments[t];
    }
    var r = e.map(function(e) {
        return Array.isArray(e) ? e : (0, CO.isArguments)(e) ? OO([], AO(e), !1) : [];
    }).reduce(function(e, t) {
        return e && e.filter ? e.filter(function(e) {
            return !(!t || !t.includes) && t.includes(e);
        }) : [];
    });
    return Array.from(new Set(r));
};

var xO = {}, FO = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(xO, "__esModule", {
    value: !0
});

var MO = FO(c_);

xO.default = function(e) {
    var t = {}, r = (0, MO.default)(e);
    return 0 === r.length || r.forEach(function(r) {
        var n = function(e) {
            var t = e;
            return null !== t && "function" != typeof t.toString && (t = toString.call(t)), 
            "".concat(t);
        }(e[r]);
        t[n] = r;
    }), t;
};

var PO = {};

Object.defineProperty(PO, "__esModule", {
    value: !0
});

var IO = fE();

PO.default = function(e) {
    return !0 === e || !1 === e || "object" == typeof e && "[object Boolean]" === (0, 
    IO.tagName)(e);
};

var kO = {};

Object.defineProperty(kO, "__esModule", {
    value: !0
});

var RO = "undefined" != typeof Buffer ? Buffer : void 0, TO = RO ? RO.isBuffer : void 0;

kO.default = function(e) {
    return Boolean(TO) && TO(e);
};

var jO = {}, LO = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(jO, "__esModule", {
    value: !0
});

var NO = fE(), BO = LO(v_);

jO.default = function(e) {
    return (0, BO.default)(e) && "[object Date]" === (0, NO.tagName)(e);
};

var UO = {}, zO = {};

!function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.getValueTag = void 0;
    var t = Symbol.toStringTag;
    e.getValueTag = function(e) {
        return t && t in Object(e) ? function(e) {
            var r = Object.prototype.hasOwnProperty.call(e, t), n = e[t], o = !1;
            try {
                e[t] = void 0, o = !0;
            } catch (e) {}
            var i = Object.prototype.toString.call(e);
            return o && (r ? e[t] = n : delete e[t]), i;
        }(e) : Object.prototype.toString.call(e);
    }, e.default = function(t) {
        if ("object" != typeof t || null == t || "[object Object]" !== (0, e.getValueTag)(t)) {
            return !1;
        }
        for (var r = Object.getPrototypeOf(t); r && null !== Object.getPrototypeOf(r); ) {
            r = Object.getPrototypeOf(r);
        }
        return Object.getPrototypeOf(t) === r;
    };
}(zO);

var HO = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(UO, "__esModule", {
    value: !0
});

var $O = HO(f_), GO = HO(kO), WO = HO(h_), VO = zO, KO = Hy;

UO.default = function(e) {
    if (null == e) {
        return !0;
    }
    if (function(e) {
        return "[object Map]" === (0, VO.getValueTag)(e) || "[object Set]" === (0, VO.getValueTag)(e);
    }(e)) {
        return 0 === e.size;
    }
    if (function(e) {
        return !!(0, $O.default)(e) && (Array.isArray(e) || "string" == typeof e || "function" == typeof e.splice || (0, 
        GO.default)(e) || (0, WO.default)(e) || (0, KO.isArguments)(e));
    }(e)) {
        return 0 === e.length;
    }
    if (function(e) {
        var t = e && e.constructor;
        return e === ("function" == typeof t && t.prototype);
    }(e)) {
        return 0 === function(e) {
            var t = [];
            return Object.keys(e).forEach(function(r) {
                Object.prototype.hasOwnProperty.call(e, r) && "constructor" !== r && t.push(r);
            }), t;
        }(e).length;
    }
    for (var t in e) {
        if (Object.prototype.hasOwnProperty.call(e, t)) {
            return !1;
        }
    }
    return !0;
};

var qO = {}, JO = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(qO, "__esModule", {
    value: !0
});

var XO = JO(ob), ZO = JO(v_), YO = JO(q_), QO = JO($y), eC = JO(h_), tC = JO(Ab), rC = JO(H_), nC = tb, oC = nb, iC = vD, aC = function(e) {
    for (var t = (0, rC.default)(e).toString(2); t.length < 64; ) {
        t = "0".concat(t);
    }
    return t;
}, sC = function(e, t) {
    if (void 0 === e) {
        return 0;
    }
    for (var r = aC(e), n = aC(t), o = 0, i = 0; i < 64; i++) {
        r[i] === n[i] && "1" === r[i] && (o = 1);
    }
    return o;
}, uC = function(e) {
    var t = 0, r = Array(e.size);
    return e.forEach(function(e, n) {
        r[t] = [ n, e ], t += 1;
    }), r.sort();
}, lC = function(e) {
    var t = 0, r = Array(e.size);
    return e.forEach(function(e) {
        r[t] = e, t += 1;
    }), r.sort();
}, cC = function(e) {
    var t, r = e.bitmask, n = e.tag, o = e.object, i = e.other, a = e.stack, s = e.customizer, u = e.equalFunc, l = r, c = sC(r, 1), f = "[object Set]" === n ? lC : uC;
    if (o.size !== i.size && !c) {
        return !1;
    }
    var d = a.get(o);
    return d ? d === i : (l = function(e, t) {
        if (void 0 === e) {
            return 1;
        }
        for (var r = aC(e), n = aC(t), o = 0, i = 0; i < 64; i++) {
            r[i] !== n[i] && (o = 1);
        }
        return o;
    }(l, 2), a.set(o, i), t = dC(f(o), f(i), l, s, u, a), a.delete(o), t);
};

function fC(e, t, r, n, o, i, a) {
    var s, u = e, l = t;
    switch (r) {
      case "[object DataView]":
        s = function(e, t) {
            return !(e.byteLength !== t.byteLength || e.byteOffset !== t.byteOffset);
        }(u, l), s && (u = u.buffer, l = l.buffer);
        break;

      case "[object ArrayBuffer]":
        s = function(e, t, r) {
            return !(e.byteLength !== t.byteLength || !r(new Uint8Array(e), new Uint8Array(t)));
        }(u, l, i);
        break;

      case "[object Map]":
      case "[object Set]":
        s = cC({
            bitmask: n,
            tag: r,
            object: u,
            other: l,
            stack: a,
            customizer: o,
            equalFunc: i
        });
        break;

      default:
        s = function(e, t, r) {
            return [ "[object Boolean]", "[object Date]", "[object Number]" ].includes(r) ? (0, 
            XO.default)(+e, +t) : [ "[object RegExp]", "[object String]" ].includes(r) ? "".concat(e) === "".concat(t) : [ "[object Error]" ].includes(r) ? e.name === t.name && e.message === t.message : !(![ "[object Symbol]" ].includes(r) || !Symbol.prototype.valueOf) && Symbol.prototype.valueOf.call(e) === Symbol.prototype.valueOf.call(t);
        }(u, l, r);
    }
    return s;
}

function dC(e, t, r, n, o, i) {
    var a = sC(r, 1), s = e.length, u = t.length;
    if (s !== u && !(a && u > s)) {
        return !1;
    }
    var l = i.get(e), c = i.get(t);
    if (l && c) {
        return l === t && c === e;
    }
    i.set(e, t), i.set(t, e);
    for (var f = 0, d = !0, p = sC(r, 2) ? new iC.SetCache([]) : void 0, h = function() {
        var a = e[f], s = t[f];
        if (f += 1, p) {
            if (!function(e, t) {
                for (var r = 0; r < (null === e ? 0 : e.length); r++) {
                    if (t(e[r], r, e)) {
                        return !0;
                    }
                }
                return !1;
            }(t, function(e, t) {
                if (!(0, oC.cacheHas)(p, t) && (a === e || o(a, e, r, n, i))) {
                    return p.push(t), p;
                }
            })) {
                return d = !1, "break";
            }
        }
        if (a !== s && !o(a, s, r, n, i)) {
            return d = !1, "break";
        }
    }; f < s; ) {
        if ("break" === h()) {
            break;
        }
    }
    return i.delete(e), i.delete(t), d;
}

var pC = function(e, t) {
    var r = e ? "[object Array]" : (0, QO.default)(t);
    return r = "[object Arguments]" === r ? "[object Object]" : r;
};

function hC(e, t, r) {
    void 0 === e && (e = void 0), void 0 === t && (t = void 0);
    var n = r.bitmask, o = r.customizer, i = r.equalFunc, a = r.stack, s = void 0 === a ? new nC.Stack(void 0) : a, u = (0, 
    YO.default)(e), l = pC(u, e);
    return l === pC((0, YO.default)(t), t) && "[object Object]" !== l ? function(e) {
        var t = e.objIsArr, r = e.object, n = e.other, o = e.objTag, i = e.bitmask, a = e.customizer, s = e.equalFunc, u = e.stack;
        return t || (0, eC.default)(r) ? dC(r, n, i, a, s, u) : fC(r, n, o, i, a, s, u);
    }({
        objIsArr: u,
        object: e,
        other: t,
        objTag: l,
        bitmask: n,
        customizer: o,
        equalFunc: i,
        stack: s
    }) : function(e, t, r, n, o, i) {
        var a = sC(r, 1), s = (0, tC.default)(e), u = (0, tC.default)(t);
        if (s.length !== u.length && !a) {
            return !1;
        }
        for (var l = s.length; l--; ) {
            var c = s[l];
            if (!(a ? c in t : Object.prototype.hasOwnProperty.hasOwnProperty.call(t, c))) {
                return !1;
            }
        }
        var f = i.get(e), d = i.get(t);
        if (f && d) {
            return f === t && d === e;
        }
        var p = !0;
        i.set(e, t), i.set(t, e);
        for (var h, v = a; ++l < s.length; ) {
            var g = e[h = s[l]], m = t[h];
            if (g !== m && !o(g, m, r, n, i)) {
                p = !1;
                break;
            }
            v = v || (v = "constructor" === h);
        }
        if (p && !v) {
            var y = e.constructor, _ = t.constructor;
            y === _ || !("constructor" in e) || !("constructor" in t) || "function" == typeof y && y instanceof y && "function" == typeof _ && _ instanceof _ || (p = !1);
        }
        return i.delete(e), i.delete(t), p;
    }(e, t, n, o, i, s);
}

function vC(e, t, r, n, o) {
    void 0 === e && (e = void 0), void 0 === t && (t = void 0);
    var i = e, a = t;
    return i === a || (null === e || null === t || !(0, ZO.default)(e) && !(0, ZO.default)(t) ? i !== e && a !== t : hC(i, a, {
        bitmask: r,
        customizer: n,
        equalFunc: vC,
        stack: o
    }));
}

qO.default = function(e, t) {
    void 0 === e && (e = void 0), void 0 === t && (t = void 0);
    try {
        return vC(e, t);
    } catch (e) {
        return !1;
    }
};

var gC = {};

Object.defineProperty(gC, "__esModule", {
    value: !0
}), gC.default = function(e) {
    return "number" == typeof e && Number.isFinite(e);
};

var mC = {};

Object.defineProperty(mC, "__esModule", {
    value: !0
});

var yC = fE();

mC.default = function(e) {
    var t = (0, yC.tagName)(e);
    return "[object Function]" === t || "[object AsyncGeneratorFunction]" === t;
};

var _C = {};

Object.defineProperty(_C, "__esModule", {
    value: !0
}), _C.default = function(e) {
    return null === e;
};

var EC = {};

Object.defineProperty(EC, "__esModule", {
    value: !0
}), EC.isPositiveInteger = EC.isNumberic = void 0;

var bC = fE();

EC.default = function(e) {
    return "number" == typeof e || "object" == typeof e && "[object Number]" === (0, 
    bC.tagName)(e);
}, EC.isNumberic = function(e) {
    return /^-?\d+(\.\d+)?$/.test(e);
}, EC.isPositiveInteger = function(e) {
    return "-0" !== e && !Object.is(-0, e) && ("0" === e || 0 === e || /^[1-9]\d*$/.test("".concat(e)));
};

var wC = {};

Object.defineProperty(wC, "__esModule", {
    value: !0
}), wC.default = function(e) {
    return void 0 === e;
};

var DC = {};

Object.defineProperty(DC, "__esModule", {
    value: !0
}), DC.default = function(e) {
    return void 0 === e && (e = void 0), Number.isInteger(e);
};

var SC = {};

Object.defineProperty(SC, "__esModule", {
    value: !0
}), SC.default = function(e) {
    return void 0 === e && (e = void 0), null != e && e instanceof Map;
};

var AC = {};

Object.defineProperty(AC, "__esModule", {
    value: !0
}), AC.default = function(e, t) {
    if (!Array.isArray(e)) {
        return "";
    }
    var r = null === t ? "null" : t;
    return r = void 0 === r ? "," : r, r = Array.isArray(r) && 0 === r.length ? "" : r, 
    r = Array.isArray(r) && r.length > 0 ? r.join(",") : r, e.join(r);
};

var OC = {};

Object.defineProperty(OC, "__esModule", {
    value: !0
}), OC.default = function(e) {
    if (null != e) {
        return 0 === e.length ? void 0 : e[e.length - 1];
    }
};

var CC = {};

Object.defineProperty(CC, "__esModule", {
    value: !0
}), CC.default = function(e) {
    var t = String(e);
    return 0 === t.length ? "" : t[0].toLowerCase() + t.substr(1);
};

var xC = {}, FC = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(xC, "__esModule", {
    value: !0
});

var MC = FC(S_), PC = FC(z_);

xC.default = function(e) {
    if (e && "number" != typeof e && e.length > 0) {
        var t = e[0];
        return e.forEach(function(e) {
            (e > t || (0, MC.default)(t) || (0, PC.default)(t)) && (t = e);
        }), t;
    }
};

var IC = {}, kC = {};

Object.defineProperty(kC, "__esModule", {
    value: !0
}), kC.default = function(e, t) {
    if (t) {
        return e.slice();
    }
    var r = e.length, n = Buffer.allocUnsafe(r);
    return e.copy(n), n;
};

var RC = {}, TC = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(RC, "__esModule", {
    value: !0
});

var jC = r_, LC = TC(KE);

RC.default = function(e) {
    return (0, jC.copyObject)(e, (0, LC.default)(e), {}, void 0);
}, function(e) {
    var t = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.baseMerge = void 0;
    var r = zy, n = n_, o = jE, i = t(kC), a = t(LE), s = t(RC), u = tb, l = Hy, c = t(ob), f = t(PS), d = t(kO), p = t(mC), h = t(qE), v = t(zO), m = t(h_), y = t(KE);
    function _(e, t) {
        if (("constructor" !== t || "function" != typeof e[t]) && "__proto__" !== t) {
            return e[t];
        }
    }
    function E(e, t, r) {
        (void 0 !== r && !(0, c.default)(e[t], r) || void 0 === r && !(t in e)) && (0, n.baseAssignValue)(e, t, r);
    }
    function b(e, t, r, n, u, c, g) {
        var y = _(e, r), b = _(t, r), w = g.get(b);
        if (w) {
            E(e, r, w);
        } else {
            var D = c ? c(y, b, "".concat(r), e, t, g) : void 0, S = void 0 === D;
            if (S) {
                var A = function(e, t) {
                    var r = !0, n = e, u = Array.isArray(e), c = (0, d.default)(e), g = !u && (0, m.default)(e);
                    return u || c || g ? Array.isArray(t) ? n = t : (0, f.default)(t) ? n = (0, a.default)(t, void 0) : c ? (r = !1, 
                    n = (0, i.default)(e, !0)) : g ? (r = !1, n = (0, o.cloneTypedArray)(e, !0)) : n = [] : (0, 
                    v.default)(e) || (0, l.isArguments)(e) ? (n = t, (0, l.isArguments)(t) ? n = (0, 
                    s.default)(t) : (0, h.default)(t) && !(0, p.default)(t) || (n = (0, o.initCloneObject)(e))) : r = !1, 
                    {
                        newValue: n,
                        isCommon: r
                    };
                }(b, y);
                D = A.newValue, S = A.isCommon;
            }
            S && (g.set(b, D), u(D, b, n, c, g), g.delete(b)), E(e, r, D);
        }
    }
    e.baseMerge = function(t, r, n, o, i) {
        if (t !== r) {
            var a = i || new u.Stack(void 0);
            (0, y.default)(r).forEach(function(i) {
                var s = r[i];
                if ((0, h.default)(s)) {
                    b(t, r, i, n, e.baseMerge, o, a);
                } else {
                    var u = o ? o(_(t, i), s, "".concat(i), t, r, a) : void 0;
                    void 0 === u && (u = s), E(t, i, u);
                }
            });
        }
    };
    var w = (0, r.createAssignFunction)(function(t, r, n) {
        return (0, e.baseMerge)(t, r, n);
    });
    e.default = w;
}(IC);

var NC = {}, BC = {}, UC = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(BC, "__esModule", {
    value: !0
});

var zC = UC(W_), HC = UC(S_), $C = UC(z_);

BC.default = function(e, t) {
    if (void 0 === t && (t = zC.default), e && "number" != typeof e && e.length > 0) {
        var r = 0, n = t(e[r]);
        return e.forEach(function(e, o) {
            var i = t(e);
            (n > i || (0, HC.default)(n) || (0, $C.default)(n)) && (n = i, r = o);
        }), e[r];
    }
};

var GC = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(NC, "__esModule", {
    value: !0
});

var WC = GC(BC);

NC.default = function(e) {
    return (0, WC.default)(e);
};

var VC = {};

Object.defineProperty(VC, "__esModule", {
    value: !0
}), VC.default = function() {};

var KC = {}, qC = {}, JC = {}, XC = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, ZC = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
};

Object.defineProperty(JC, "__esModule", {
    value: !0
}), JC.default = function(e) {
    for (var t = [], r = 1; r < arguments.length; r++) {
        t[r - 1] = arguments[r];
    }
    if (null == e) {
        return [];
    }
    var n = [];
    t.forEach(function(e) {
        Array.isArray(e) ? n.push.apply(n, ZC([], XC(e), !1)) : n.push(e);
    });
    for (var o = e.length - 1; o >= 0; o--) {
        n.includes(e[o]) && e.splice(o, 1);
    }
    return e;
};

var YC = {}, QC = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(YC, "__esModule", {
    value: !0
});

var ex = QC(TA), tx = QC(dE());

YC.default = function e(t) {
    return null == t ? "" : (0, ex.default)(t) ? t : Array.isArray(t) ? "".concat(t.map(function(t) {
        return null == t ? t : e(t);
    })) : (0, tx.default)(t) ? t.toString() : "0" === "".concat(t) && 1 / t == -1 / 0 ? "-0" : "".concat(t);
};

var rx = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, nx = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
}, ox = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(qC, "__esModule", {
    value: !0
}), qC.checkIsNestedObject = qC.getNestedValue = qC.getObjectKeys = qC.getFilters = void 0;

var ix = ox(EC), ax = ox(S_), sx = ox(qE), ux = ox(JC), lx = ox(YC);

function cx(e) {
    for (var t = [], r = 0, n = e.length; r < n; r++) {
        var o = e[r];
        Array.isArray(o) ? t = t.concat(o) : "string" == typeof o || (0, ix.default)(o) ? t.push((0, 
        lx.default)(o)) : "[object Symbol]" === Object.prototype.toString.call(o) ? t.push(o) : "[object Arguments]" === Object.prototype.toString.call(o) && t.push.apply(t, nx([], rx(o), !1));
    }
    return t;
}

function fx(e) {
    var t = Object.keys(e), r = Object.getOwnPropertySymbols(e);
    return r.length > 0 ? [].concat(t).concat(r) : t;
}

function dx(e, t, r, n, o) {
    void 0 === o && (o = !1);
    for (var i = [].concat(r), a = 0, s = t.length; a < s; a++) {
        var u = t[a];
        i.includes(u) && (n[u] = e[u], o && (0, ux.default)(i, [ u ]));
    }
    return i;
}

function px(e, t) {
    var r = {}, n = "[object Symbol]" === Object.prototype.toString.call(t) ? [ t ] : t.split(".");
    if (n.length > 1) {
        var o = n.shift();
        if ((0, ax.default)(e[o])) {
            return r;
        }
        var i = px(e[o], n.join("."));
        return r[o] = i, r;
    }
    return Object.prototype.hasOwnProperty.call(e, t) && (r[t] = e[t]), r;
}

function hx(e, t) {
    if (!(0, sx.default)(e) && !(0, sx.default)(t)) {
        return t;
    }
    for (var r = Object.keys(t), n = 0, o = r.length; n < o; n++) {
        var i = r[n];
        e[i] = e[i] ? hx(e[i], t[i]) : t[i];
    }
    return e;
}

function vx(e, t, r) {
    if (e.length > 0) {
        for (var n = 0, o = e.length; n < o; n++) {
            var i = px(t, e[n]), a = Object.keys(i)[0];
            Object.prototype.hasOwnProperty.call(r, a) ? hx(r, i) : Object.assign(r, i);
        }
    }
}

function gx(e) {
    for (var t = Object.keys(e), r = 0, n = t.length; r < n; r++) {
        if ("[object Object]" === Object.prototype.toString.call(e[t[r]])) {
            return !0;
        }
    }
    return !1;
}

qC.getFilters = cx, qC.getObjectKeys = fx, qC.getNestedValue = px, qC.checkIsNestedObject = gx, 
qC.default = function(e) {
    for (var t = [], r = 1; r < arguments.length; r++) {
        t[r - 1] = arguments[r];
    }
    if ((0, ax.default)(e)) {
        return {};
    }
    var n = cx(t);
    return gx(e) ? function(e, t) {
        var r = {}, n = dx(e, fx(e), t, r, !0);
        vx(n, e, r);
        var o = Object.getPrototypeOf(e), i = Object.keys(o);
        return gx(o) ? vx(n = dx(o, i, t, r, !0), o, r) : dx(o, i, t, r), r;
    }(e, n) : function(e, t) {
        var r = {};
        dx(e, fx(e), t, r);
        var n = Object.getPrototypeOf(e);
        return vx(dx(n, Object.keys(n), t, r, !0), n, r), r;
    }(e, n);
};

var mx = g && g.__values || function(e) {
    var t = "function" == typeof Symbol && Symbol.iterator, r = t && e[t], n = 0;
    if (r) {
        return r.call(e);
    }
    if (e && "number" == typeof e.length) {
        return {
            next: function() {
                return e && n >= e.length && (e = void 0), {
                    value: e && e[n++],
                    done: !e
                };
            }
        };
    }
    throw new TypeError(t ? "Object is not iterable." : "Symbol.iterator is not defined.");
}, yx = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, _x = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
}, Ex = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(KC, "__esModule", {
    value: !0
});

var bx = Ex(N_), wx = Ex(uE()), Dx = fE(), Sx = Ex(UO), Ax = qC, Ox = Ex(YC);

function Cx(e, t) {
    var r, n;
    try {
        for (var o = mx(t), i = o.next(); !i.done; i = o.next()) {
            if (e === i.value) {
                return !0;
            }
        }
    } catch (e) {
        r = {
            error: e
        };
    } finally {
        try {
            i && !i.done && (n = o.return) && n.call(o);
        } finally {
            if (r) {
                throw r.error;
            }
        }
    }
    return !1;
}

function xx(e, t, r) {
    if (0 === r.length) {
        return (0, bx.default)({}, e);
    }
    var n = {}, o = (0, Ax.getObjectKeys)(e);
    return r.forEach(function(i) {
        var a = 1;
        "string" == typeof i && (a = (0, wx.default)(i).length), o.forEach(function(o) {
            var i = "".concat((0, Ox.default)(t), ".").concat((0, Ox.default)(o));
            if (!Cx(i, r) && Object.getOwnPropertyDescriptor(e, o).enumerable) {
                if ("[object Object]" === Object.prototype.toString.call(e[o])) {
                    var s = Fx(e[o], i, r, a);
                    (0, Sx.default)(s) || (n[o] = s);
                } else {
                    n[o] = e[o];
                }
            }
        });
    }), n;
}

function Fx(e, t, r, n) {
    var o = {};
    return (0, Ax.getObjectKeys)(e).forEach(function(i) {
        var a = "".concat((0, Ox.default)(t), ".").concat((0, Ox.default)(i));
        if (!Cx(a, r) && Object.getOwnPropertyDescriptor(e, i).enumerable) {
            if ("[object Object]" === Object.prototype.toString.call(e[i]) && 1 !== n) {
                var s = Fx(e[i], a, r, n - 1);
                (0, Sx.default)(s) || (o[i] = s);
            } else {
                o[i] = e[i];
            }
        }
    }), o;
}

function Mx(e, t, r) {
    var n, o, i = new Map, a = (0, Ax.getObjectKeys)(e), s = [].concat.apply([], _x([], yx(t), !1)), u = function(n) {
        return Cx(n, t) ? (s = s.filter(function(e) {
            return e !== n;
        }), "continue") : Object.getOwnPropertyDescriptor(e, n).enumerable ? void ("[object Object]" !== (0, 
        Dx.tagName)(e[n]) ? r[n] = e[n] : i.set(n, e[n])) : "continue";
    };
    try {
        for (var l = mx(a), c = l.next(); !c.done; c = l.next()) {
            u(c.value);
        }
    } catch (e) {
        n = {
            error: e
        };
    } finally {
        try {
            c && !c.done && (o = l.return) && o.call(l);
        } finally {
            if (n) {
                throw n.error;
            }
        }
    }
    i.forEach(function(e, t) {
        var n = xx(e, t, s);
        (0, Sx.default)(n) || (r[t] = n);
    });
}

KC.default = function(e) {
    for (var t = [], r = 1; r < arguments.length; r++) {
        t[r - 1] = arguments[r];
    }
    if (null == e) {
        return {};
    }
    var n = (0, Ax.getFilters)(t);
    return (0, Ax.checkIsNestedObject)(e) ? function(e, t) {
        var r = {};
        Mx(e, t, r);
        var n = Object.getPrototypeOf(e);
        return null !== n && Mx(n, t, r), r;
    }(e, n) : function(e, t) {
        var r = {};
        Mx(e, t, r);
        var n = Object.getPrototypeOf(e);
        return Array.isArray(n) || null === n || Mx(n, t, r), r;
    }(e, n);
};

var Px = {}, Ix = {}, kx = {};

Object.defineProperty(kx, "__esModule", {
    value: !0
}), kx.default = function(e, t) {
    if (null == e) {
        return e;
    }
    for (var r = 1, n = t.length, o = e[t[0]]; null != o && r < n; ) {
        o = o[t[r]], r += 1;
    }
    return o;
};

var Rx, Tx = {}, jx = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Tx, "__esModule", {
    value: !0
});

var Lx = jx(dE()), Nx = ((Rx = {}).boolean = 0, Rx.number = 1, Rx.string = 2, Rx[typeof Symbol("a")] = 3, 
Rx.object = 4, Rx[void 0] = 5, Rx);

Tx.default = function(e, t) {
    var r, n, o = typeof e, i = typeof t, a = Number.isNaN(e), s = Number.isNaN(t);
    return o !== i || a || s ? (r = a ? 6 : Nx[o], n = s ? 6 : Nx[i]) : (r = e, n = t, 
    (0, Lx.default)(e) && (r = e.description, n = t.description)), function(e, t) {
        return e > t ? 1 : e < t ? -1 : 0;
    }(r, n);
};

var Bx = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Ix, "__esModule", {
    value: !0
}), Ix.arraySort = void 0;

var Ux = Bx(kx), zx = Bx(W_), Hx = Bx(Tx);

Ix.arraySort = function(e, t, r) {
    var n;
    n = t.length > 0 ? t.map(function(e) {
        return Array.isArray(e) ? function(t) {
            return (0, Ux.default)(t, e);
        } : "function" == typeof e ? e : "object" == typeof e || "string" == typeof e ? function(t) {
            return (0, Ux.default)(t, [ e ]);
        } : zx.default;
    }) : [ zx.default ];
    for (var o = [], i = e.length, a = 0; a < i; a++) {
        o.push({
            value: e[a],
            index: a
        });
    }
    o.sort(function(e, t) {
        return function(e) {
            for (var t = e.length, r = 0; r < t; r++) {
                if (0 !== e[r]) {
                    return e[r];
                }
            }
            return 0;
        }(n.map(function(n, o) {
            var i, a = r[o] ? r[o] : "asc";
            if ("function" == typeof a) {
                i = a(n(e.value), n(t.value));
            } else {
                var s = n(e.value), u = n(t.value);
                i = (0, Hx.default)(s, u);
            }
            return "desc" === a.toString() && (i = 0 - i), i;
        }));
    });
    var s = [];
    for (a = 0; a < i; a++) {
        s[a] = o[a].value;
    }
    return s;
};

var $x = g && g.__values || function(e) {
    var t = "function" == typeof Symbol && Symbol.iterator, r = t && e[t], n = 0;
    if (r) {
        return r.call(e);
    }
    if (e && "number" == typeof e.length) {
        return {
            next: function() {
                return e && n >= e.length && (e = void 0), {
                    value: e && e[n++],
                    done: !e
                };
            }
        };
    }
    throw new TypeError(t ? "Object is not iterable." : "Symbol.iterator is not defined.");
};

Object.defineProperty(Px, "__esModule", {
    value: !0
});

var Gx = Ix;

Px.default = function(e, t, r) {
    return null == e ? [] : function(e, t, r) {
        if (Array.isArray(e)) {
            var n = [].concat(e);
            return (0, Gx.arraySort)(n, t, r);
        }
        return function(e, t, r) {
            var n, o, i = [], a = Object.keys(e);
            try {
                for (var s = $x(a), u = s.next(); !u.done; u = s.next()) {
                    var l = u.value;
                    i.push(e[l]);
                }
            } catch (e) {
                n = {
                    error: e
                };
            } finally {
                try {
                    u && !u.done && (o = s.return) && o.call(s);
                } finally {
                    if (n) {
                        throw n.error;
                    }
                }
            }
            return (0, Gx.arraySort)(i, t, r);
        }(e, t, r);
    }(e, Array.isArray(t) ? t : [ t ], Array.isArray(r) ? r : null == r ? [] : [ r ]);
};

var Wx = {}, Vx = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Wx, "__esModule", {
    value: !0
});

var Kx = Vx(H_), qx = Vx(YC);

Wx.default = function(e, t, r) {
    var n = (0, qx.default)(e), o = (0, Kx.default)(t), i = void 0 === r ? " " : (0, 
    qx.default)(r);
    if (o <= n.length || "" === i) {
        return n;
    }
    for (var a = "", s = 0; s < o - n.length && !("".concat(a += i).concat(n).length >= o); s++) {}
    if ("".concat(n).concat(a).length > o) {
        var u = "".concat(n).concat(a).length - o;
        a = a.substring(0, a.length - u);
    }
    return "".concat(n).concat(a);
};

var Jx = {}, Xx = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Jx, "__esModule", {
    value: !0
});

var Zx = Xx(H_), Yx = Xx(YC);

Jx.default = function(e, t, r) {
    var n = (0, Yx.default)(e), o = (0, Zx.default)(t), i = void 0 === r ? " " : (0, 
    Yx.default)(r);
    if (o <= n.length || "" === i) {
        return n;
    }
    for (var a = "", s = 0; s < t - n.length && !("".concat(a += i).concat(n).length >= t); s++) {}
    if ("".concat(a).concat(n).length > t) {
        var u = "".concat(a).concat(n).length - t;
        a = a.substring(0, a.length - u);
    }
    return "".concat(a).concat(n);
};

var Qx = {}, eF = g && g.__values || function(e) {
    var t = "function" == typeof Symbol && Symbol.iterator, r = t && e[t], n = 0;
    if (r) {
        return r.call(e);
    }
    if (e && "number" == typeof e.length) {
        return {
            next: function() {
                return e && n >= e.length && (e = void 0), {
                    value: e && e[n++],
                    done: !e
                };
            }
        };
    }
    throw new TypeError(t ? "Object is not iterable." : "Symbol.iterator is not defined.");
};

Object.defineProperty(Qx, "__esModule", {
    value: !0
});

var tF = fE();

Qx.default = function(e) {
    for (var t, r, n = [], o = 1; o < arguments.length; o++) {
        n[o - 1] = arguments[o];
    }
    if (!e) {
        return [];
    }
    var i = [];
    try {
        for (var a = eF(n), s = a.next(); !s.done; s = a.next()) {
            var u = s.value;
            Array.isArray(u) ? i = i.concat(u.map(function(e) {
                return (0, tF.toStringWithZeroSign)(e);
            })) : i.push((0, tF.toStringWithZeroSign)(u));
        }
    } catch (e) {
        t = {
            error: e
        };
    } finally {
        try {
            s && !s.done && (r = a.return) && r.call(a);
        } finally {
            if (t) {
                throw t.error;
            }
        }
    }
    if (0 === i.length) {
        return [];
    }
    var l = [], c = new Map;
    if (i.forEach(function(t) {
        var r;
        if (c.has(t)) {
            r = c.get(t);
        } else {
            var n = function(e, t) {
                var r, n;
                if (Object.prototype.hasOwnProperty.call(e, t)) {
                    return {
                        penultimateValue: e,
                        lastKey: t
                    };
                }
                var o = t.split("."), i = o.pop(), a = e;
                try {
                    for (var s = eF(o), u = s.next(); !u.done; u = s.next()) {
                        var l = u.value, c = a[l];
                        if (null == c) {
                            i = l;
                            break;
                        }
                        a = c;
                    }
                } catch (e) {
                    r = {
                        error: e
                    };
                } finally {
                    try {
                        u && !u.done && (n = s.return) && n.call(s);
                    } finally {
                        if (r) {
                            throw r.error;
                        }
                    }
                }
                return {
                    penultimateValue: a,
                    lastKey: i
                };
            }(e, t), o = n.penultimateValue, i = n.lastKey;
            r = o[i], delete o[i];
        }
        l.push(r), c.has(t) || c.set(t, r);
    }), Array.isArray(e)) {
        for (var f = [], d = 0; d < e.length; d++) {
            Object.prototype.hasOwnProperty.call(e, String(d)) && f.push(e[d]);
        }
        f.forEach(function(t, r) {
            e[r] = t;
        }), e.length = f.length;
    }
    return l;
};

var rF = {}, nF = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(rF, "__esModule", {
    value: !0
});

var oF = nF(z_);

rF.default = function() {
    for (var e = [], t = 0; t < arguments.length; t++) {
        e[t] = arguments[t];
    }
    for (var r = function(e) {
        var t = 0, r = (0, oF.default)(Number(e[0])) ? 0 : Number(e[0]), n = r < 0 ? -1 : 1;
        return 1 === e.length ? {
            start: t,
            end: r,
            step: n
        } : {
            start: t = (0, oF.default)(Number(e[0])) ? 0 : Number(e[0]),
            end: r = (0, oF.default)(Number(e[1])) ? 0 : Number(e[1]),
            step: n = 2 === e.length ? r > t ? 1 : -1 : (0, oF.default)(Number(e[2])) ? 0 : Number(e[2])
        };
    }(e), n = r.start, o = r.end, i = r.step, a = o < n && i > 0, s = Math.abs(Math.ceil((o - n) / (i || 1))), u = new Array(s), l = n, c = 0; c < s; c++) {
        u[c] = l, l = a ? l - i : l + i;
    }
    return u;
};

var iF = {}, aF = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(iF, "__esModule", {
    value: !0
});

var sF = aF(S_), uF = aF(c_), lF = function(e) {
    return e;
};

iF.default = function(e, t, r) {
    void 0 === t && (t = lF);
    var n = arguments.length < 3;
    return Array.isArray(e) ? function(e, t, r, n) {
        var o = (0, sF.default)(e) ? 0 : e.length, i = 0, a = n;
        r && o > 0 && (a = e[0], i = 1);
        for (var s = i; s < o; s++) {
            a = t(a, e[s], s, e);
        }
        return a;
    }(e, t, n, r) : function(e, t, r, n) {
        var o = (0, uF.default)(e), i = o.length, a = 0, s = n;
        r && i > 0 && (s = e[o[0]], a = 1);
        for (var u = a; u < i; u++) {
            var l = o[u];
            s = t(s, e[l], l, e);
        }
        return s;
    }(e, t, n, r);
};

var cF = {};

Object.defineProperty(cF, "__esModule", {
    value: !0
});

var fF = fA;

cF.default = function(e, t) {
    if (null == e) {
        return [];
    }
    for (var r = [], n = (0, fF.wrapIteratee)(t), o = 0, i = e.length; o < i; o++) {
        n(e[o], o, e) && r.push(e[o]);
    }
    for (o = e.length - 1; o >= 0; o--) {
        r.includes(e[o]) && e.splice(o, 1);
    }
    return r;
};

var dF = {};

Object.defineProperty(dF, "__esModule", {
    value: !0
}), dF.default = function(e) {
    for (var t = e.length, r = 0; r < t / 2; r++) {
        var n = t - r - 1, o = e[r];
        e[r] = e[n], e[n] = o;
    }
    return e;
};

var pF = {}, hF = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(pF, "__esModule", {
    value: !0
});

var vF = hF(GS), gF = hF(H_), mF = hF(YC);

pF.default = function(e, t) {
    var r = (0, vF.default)(t), n = (0, gF.default)(e);
    if (r) {
        var o = "".concat((0, mF.default)(n), "e").split("e"), i = Math.round("".concat(o[0], "e").concat(+o[1] + r));
        return o = "".concat((0, mF.default)(i), "e").split("e"), +"".concat(o[0], "e").concat(+o[1] - r);
    }
    return Math.round(n);
};

var yF = {}, _F = g && g.__createBinding || (Object.create ? function(e, t, r, n) {
    void 0 === n && (n = r);
    var o = Object.getOwnPropertyDescriptor(t, r);
    o && !("get" in o ? !t.__esModule : o.writable || o.configurable) || (o = {
        enumerable: !0,
        get: function() {
            return t[r];
        }
    }), Object.defineProperty(e, n, o);
} : function(e, t, r, n) {
    void 0 === n && (n = r), e[n] = t[r];
}), EF = g && g.__setModuleDefault || (Object.create ? function(e, t) {
    Object.defineProperty(e, "default", {
        enumerable: !0,
        value: t
    });
} : function(e, t) {
    e.default = t;
}), bF = g && g.__importStar || function(e) {
    if (e && e.__esModule) {
        return e;
    }
    var t = {};
    if (null != e) {
        for (var r in e) {
            "default" !== r && Object.prototype.hasOwnProperty.call(e, r) && _F(t, e, r);
        }
    }
    return EF(t, e), t;
}, wF = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(yF, "__esModule", {
    value: !0
});

var DF = wF(TA), SF = bF(EC), AF = wF(uE()), OF = wF(H_);

function CF(e, t, r) {
    return e ? t ? [] : {} : r ? [] : {};
}

function xF(e, t, r) {
    var n;
    if (FF(e, t)) {
        kF(e[t]) && (e[t] = r ? [] : {}), n = e[t];
    } else if ((0, DF.default)(t)) {
        for (var o = e, i = (0, AF.default)(t), a = i.length, s = 0; s < a; s++) {
            var u = i[s];
            FF(o, u) && !kF(o[u]) || (o[u] = CF(IF(s, a), r, (0, SF.isPositiveInteger)(i[s + 1]))), 
            o = o[u];
        }
        n = o;
    } else {
        e[t] = r ? [] : {}, n = e[t];
    }
    return n;
}

function FF(e, t) {
    var r = Object.prototype.hasOwnProperty;
    return null != e[t] || r.apply(e, [ t ]);
}

function MF(e, t, r) {
    if (FF(e, t)) {
        (0, SF.isPositiveInteger)(t) ? e[(0, OF.default)(t)] = r : (0, SF.default)(t) ? PF(e, t.toLocaleString(), r) : PF(e, t, r);
    } else {
        for (var n = (0, AF.default)(t), o = n.length, i = e, a = 0; a < o; a++) {
            var s = n[a];
            if (IF(a, o)) {
                PF(i, (0, SF.isPositiveInteger)(s) ? (0, OF.default)(s) : s, r);
            } else {
                var u = (0, SF.isPositiveInteger)(s) ? (0, OF.default)(s) : s, l = i[u];
                if (kF(l)) {
                    l = (0, SF.isPositiveInteger)(n[a + 1]) ? [] : {}, i[u] = l;
                }
                i = i[u];
            }
        }
    }
}

function PF(e, t, r) {
    Number.isNaN(e[t]) && Number.isNaN(r) || e[t] === r || (e[t] = r);
}

function IF(e, t) {
    return e + 1 === t;
}

function kF(e) {
    var t = typeof e;
    return null == e || "string" === t || "number" === t || "boolean" === t || "symbol" === t || "bigint" === t;
}

yF.default = function(e, t, r) {
    return null == e ? e : function(e, t, r) {
        var n;
        n = Array.isArray(t) ? t : [ t ];
        var o = e, i = 0, a = n.length;
        for (;i < a; ) {
            var s = n[i];
            if (i === a - 1) {
                MF(o, s, r);
                break;
            }
            o = xF(o, s, (0, SF.isPositiveInteger)(n[i + 1])), i += 1;
        }
        return e;
    }(e, t, r);
};

var RF = {};

Object.defineProperty(RF, "__esModule", {
    value: !0
}), RF.default = function(e, t, r) {
    var n = Number.isNaN, o = null == e ? 0 : e.length;
    if (!o || n(r)) {
        return [];
    }
    var i = Math.floor, a = null == t || n(t) ? 0 : i(Number(t)), s = void 0 === r ? o : i(Number(r));
    a < 0 && (a = -a > o ? 0 : o + a), (s = s > o ? o : s) < 0 && (s += o), o = a > s ? 0 : s - a >>> 0, 
    a >>>= 0;
    for (var u = -1, l = []; ++u < o; ) {
        l[u] = e[u + a];
    }
    return l;
};

var TF = {}, jF = g && g.__values || function(e) {
    var t = "function" == typeof Symbol && Symbol.iterator, r = t && e[t], n = 0;
    if (r) {
        return r.call(e);
    }
    if (e && "number" == typeof e.length) {
        return {
            next: function() {
                return e && n >= e.length && (e = void 0), {
                    value: e && e[n++],
                    done: !e
                };
            }
        };
    }
    throw new TypeError(t ? "Object is not iterable." : "Symbol.iterator is not defined.");
}, LF = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, NF = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
}, BF = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(TF, "__esModule", {
    value: !0
});

var UF = Ix, zF = BF(W_);

TF.default = function(e) {
    for (var t, r, n = [], o = 1; o < arguments.length; o++) {
        n[o - 1] = arguments[o];
    }
    var i = [].concat(e);
    if (!Array.isArray(e)) {
        var a = Object.keys(e);
        i = [];
        try {
            for (var s = jF(a), u = s.next(); !u.done; u = s.next()) {
                var l = u.value;
                i.push(e[l]);
            }
        } catch (e) {
            t = {
                error: e
            };
        } finally {
            try {
                u && !u.done && (r = s.return) && r.call(s);
            } finally {
                if (t) {
                    throw t.error;
                }
            }
        }
    }
    var c = [];
    if (null == n || 0 === n.length) {
        c = [ zF.default ];
    } else {
        for (var f = n.length, d = 0; d < f; d++) {
            Array.isArray(n[d]) ? c.push.apply(c, NF([], LF(n[d]), !1)) : null == n[d] ? c.push(zF.default) : c.push(n[d]);
        }
    }
    return (0, UF.arraySort)(i, c, [ "asc" ]);
};

var HF = {};

Object.defineProperty(HF, "__esModule", {
    value: !0
}), HF.default = function(e, t, r) {
    var n = null == r ? 0 : r;
    return (n < 0 || Number.isNaN(n)) && (n = 0), n > e.length && (n = e.length), n >= 0 && e.slice(n, n + t.length) === t;
};

var $F = {};

Object.defineProperty($F, "__esModule", {
    value: !0
}), $F.default = function(e) {
    if (null == e) {
        return 0;
    }
    var t = 0;
    try {
        for (var r = 0, n = e.length; r < n; r++) {
            void 0 !== e[r] && (t += e[r]);
        }
    } catch (e) {}
    return "string" == typeof t ? "".concat(t).substring(1, "".concat(t).length) : t;
};

var GF = {}, WF = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(GF, "__esModule", {
    value: !0
});

var VF = WF(Lw);

GF.default = function(e, t, r) {
    if (void 0 === r && (r = {}), "function" != typeof e) {
        throw new TypeError("Expected a function");
    }
    var n = r.leading, o = void 0 === n || n, i = r.trailing, a = void 0 === i || i;
    return (0, VF.default)(e, t, {
        leading: o,
        trailing: a,
        maxWait: t
    });
};

var KF = {}, qF = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(KF, "__esModule", {
    value: !0
});

var JF = qF(S_), XF = String ? String.prototype.toLowerCase : void 0;

KF.default = function(e) {
    return void 0 === e && (e = ""), (0, JF.default)(e) ? "" : XF.call(e);
};

var ZF = {}, YF = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(ZF, "__esModule", {
    value: !0
});

var QF = YF(S_), eM = String ? String.prototype.toUpperCase : void 0;

ZF.default = function(e) {
    return (0, QF.default)(e) ? e : eM.call(e);
};

var tM = {};

Object.defineProperty(tM, "__esModule", {
    value: !0
});

var rM = fE();

tM.default = function(e, t) {
    if (null == e) {
        return "";
    }
    for (var r = t ? t.split("") : rM.whiteSpace, n = e.split(""), o = -1, i = n.length - 1; i >= 0; i--) {
        if (!r.includes(n[i])) {
            o = i;
            break;
        }
    }
    return e.substring(0, o + 1);
};

var nM = {};

Object.defineProperty(nM, "__esModule", {
    value: !0
});

var oM = fE();

nM.default = function(e, t) {
    if (null == e) {
        return "";
    }
    for (var r = t ? t.split("") : oM.whiteSpace, n = e.split(""), o = n.length, i = 0; i < o; i++) {
        if (!r.includes(n[i])) {
            o = i;
            break;
        }
    }
    return e.substring(o, e.length);
};

var iM = {};

Object.defineProperty(iM, "__esModule", {
    value: !0
}), iM.default = function(e) {
    if (!Array.isArray(e)) {
        return [];
    }
    for (var t = [], r = 0; r < e.length; r++) {
        var n = "number" == typeof e[r] ? e[r] + 0 : e[r];
        t.includes(n) || t.push(n);
    }
    return t;
};

var aM = {}, sM = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(aM, "__esModule", {
    value: !0
});

var uM = sM(S_), lM = sM(ZF);

aM.default = function(e) {
    return void 0 === e && (e = ""), (0, uM.default)(e) || 0 === e.length ? e : (0, 
    lM.default)(e[0]) + e.slice(1);
};

var cM = {}, fM = {}, dM = {}, pM = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(dM, "__esModule", {
    value: !0
});

var hM = pM(z_);

dM.default = function(e, t) {
    return (0, hM.default)(e) && (0, hM.default)(t) || e === t;
};

var vM = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(fM, "__esModule", {
    value: !0
});

var gM = vM(iF), mM = vM(f_), yM = vM(oA), _M = vM(dM), EM = vM(OC), bM = vM(mC);

fM.default = function() {
    for (var e = [], t = 0; t < arguments.length; t++) {
        e[t] = arguments[t];
    }
    var r = [ [] ].concat(e), n = r, o = (0, EM.default)(r);
    return (0, bM.default)(o) ? n.pop() : o = _M.default, (0, gM.default)(n, function(e, t) {
        return (0, mM.default)(t) ? ((0, yM.default)(t, function(t) {
            -1 === e.findIndex(function(e) {
                return o(t, e);
            }) && e.push(t);
        }), e) : e;
    });
};

var wM = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, DM = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
}, SM = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(cM, "__esModule", {
    value: !0
});

var AM = SM(fM);

cM.default = function() {
    for (var e = [], t = 0; t < arguments.length; t++) {
        e[t] = arguments[t];
    }
    return AM.default.apply(void 0, DM([], wM(e), !1));
};

var OM = {}, CM = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(OM, "__esModule", {
    value: !0
});

var xM = CM(S_), FM = {};

OM.default = function(e) {
    void 0 === e && (e = ""), (0, xM.default)(FM["".concat(e)]) && (FM["".concat(e)] = 0), 
    FM["".concat(e)] += 1;
    var t = FM["".concat(e)];
    return "$lodash$" === "".concat(e) ? "".concat(t) : "".concat(e).concat(t);
};

var MM = {}, PM = {}, IM = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, kM = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
};

Object.defineProperty(PM, "__esModule", {
    value: !0
}), PM.default = function() {
    for (var e = [], t = 0; t < arguments.length; t++) {
        e[t] = arguments[t];
    }
    var r = e, n = function(e, t) {
        return e === t;
    }, o = e.length, i = e[o - 1];
    if ("function" == typeof i && (r = e.slice(0, o - 1), n = i), !r || 0 === r.length) {
        return [];
    }
    var a = r.filter(function(e) {
        return Array.isArray(e) || "[object Arguments]" === Object.prototype.toString.call(e);
    }).map(function(e) {
        var t = kM([], IM(e), !1), r = [];
        return t.forEach(function(e) {
            r.findIndex(function(t) {
                return n(t, e);
            }) < 0 && r.push(e);
        }), r;
    }).reduce(function(e, t) {
        return kM(kM([], IM(e), !1), IM(t), !1);
    }), s = [];
    return a.forEach(function(e, t) {
        var r = kM([], IM(a), !1);
        r.splice(t, 1), r.findIndex(function(t) {
            return n(t, e);
        }) < 0 && s.push(e);
    }), s;
};

var RM = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, TM = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
}, jM = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(MM, "__esModule", {
    value: !0
});

var LM = jM(ob), NM = jM(PM);

MM.default = function() {
    for (var e = [], t = 0; t < arguments.length; t++) {
        e[t] = arguments[t];
    }
    return NM.default.apply(void 0, TM(TM([], RM(e), !1), [ function(e, t) {
        return (0, LM.default)(e, t);
    } ], !1));
};

var BM = {};

Object.defineProperty(BM, "__esModule", {
    value: !0
}), BM.default = function(e, t) {
    var r = e;
    "[object Object]" === Object.prototype.toString.call(e) && (r = Object.keys(e).map(function(t) {
        return e[t];
    }));
    var n = {}, o = function(e) {
        return e;
    }, i = typeof t;
    return "function" === i ? o = function(e) {
        return t(e);
    } : "string" !== i && "number" !== i || (o = function(e) {
        return e[t];
    }), r.forEach(function(e) {
        n[o(e)] = e;
    }), n;
};

var UM = {}, zM = g && g.__values || function(e) {
    var t = "function" == typeof Symbol && Symbol.iterator, r = t && e[t], n = 0;
    if (r) {
        return r.call(e);
    }
    if (e && "number" == typeof e.length) {
        return {
            next: function() {
                return e && n >= e.length && (e = void 0), {
                    value: e && e[n++],
                    done: !e
                };
            }
        };
    }
    throw new TypeError(t ? "Object is not iterable." : "Symbol.iterator is not defined.");
}, HM = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(UM, "__esModule", {
    value: !0
});

var $M = HM(S_);

UM.default = function(e, t) {
    var r, n;
    if ((0, $M.default)(e)) {
        return e;
    }
    var o = Object.keys(e);
    try {
        for (var i = zM(o), a = i.next(); !a.done; a = i.next()) {
            var s = a.value;
            if (!1 === t(e[s], s)) {
                break;
            }
        }
    } catch (e) {
        r = {
            error: e
        };
    } finally {
        try {
            a && !a.done && (n = i.return) && n.call(i);
        } finally {
            if (r) {
                throw r.error;
            }
        }
    }
    return e;
};

var GM = {}, WM = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(GM, "__esModule", {
    value: !0
});

var VM = WM(S_), KM = WM(dM);

GM.default = function(e) {
    for (var t = [], r = 1; r < arguments.length; r++) {
        t[r - 1] = arguments[r];
    }
    return (0, VM.default)(t) ? [].concat(e) : e.filter(function(e) {
        return -1 === t.findIndex(function(t) {
            return (0, KM.default)(e, t);
        });
    });
};

var qM = {}, JM = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(qM, "__esModule", {
    value: !0
});

var XM = JM(dO).default;

qM.default = XM;

var ZM = {}, YM = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(ZM, "__esModule", {
    value: !0
});

var QM = YM(S_), eP = YM(DC), tP = YM(z_);

ZM.default = function(e, t) {
    if (void 0 === t && (t = 0), !(0, QM.default)(e) && 0 !== e.length) {
        var r = t;
        if ((0, eP.default)(t) || (r = Number.parseInt(t.toString(), 10)), !(0, tP.default)(r)) {
            var n = e.length;
            return r < 0 && (r += n), e[r];
        }
    }
};

var rP = {}, nP = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(rP, "__esModule", {
    value: !0
});

var oP = nP(S_), iP = nP(dM);

function aP(e) {
    return (0, oP.default)(e) || 0 === e.length;
}

function sP(e, t, r) {
    for (var n = r - 1, o = e.length; ++n < o; ) {
        if ((0, iP.default)(e[n], t)) {
            return n;
        }
    }
    return -1;
}

rP.default = function(e, t) {
    if (aP(e) || aP(t)) {
        return e;
    }
    if (e === t) {
        return e.length = 0, e;
    }
    for (var r = t.length, n = -1; ++n < r; ) {
        for (var o = t[n], i = 0; (i = sP(e, o, i)) > -1; ) {
            e.splice(i, 1);
        }
    }
    return e;
};

var uP = {}, lP = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(uP, "__esModule", {
    value: !0
});

var cP = lP(W_), fP = lP(z_), dP = lP(S_);

uP.default = function(e, t) {
    if (void 0 === e && (e = 0), e <= 0 || e === 1 / 0 || (0, fP.default)(e)) {
        return [];
    }
    var r = t;
    (0, dP.default)(r) && (r = cP.default);
    for (var n = Math.floor(e), o = [], i = 0; i < n; i++) {
        o.push(r(i));
    }
    return o;
};

var pP = {}, hP = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(pP, "__esModule", {
    value: !0
});

var vP = hP(DC), gP = hP(z_), mP = hP(S_), yP = hP(H_);

pP.default = function(e, t, r) {
    var n = r, o = e, i = t;
    (0, mP.default)(r) && ("boolean" == typeof t ? (n = t, i = void 0) : "boolean" == typeof e && (n = e, 
    o = void 0)), o = (0, mP.default)(o) ? 0 : o, i = (0, mP.default)(i) ? 1 : i, "number" != typeof o && (o = (0, 
    yP.default)(o)), "number" != typeof i && (i = (0, yP.default)(i)), (0, gP.default)(i) && (i = 0), 
    (0, gP.default)(o) && (o = 0);
    var a = o;
    return o > i && (o = i, i = a), o === 1 / 0 || i === 1 / 0 ? Number.MAX_VALUE : ("boolean" != typeof n && (n = !1), 
    function(e, t, r) {
        return r || !(0, vP.default)(e) || !(0, vP.default)(t);
    }(o, i, n) ? Math.random() * (i - o) + o : function(e, t) {
        var r = Math.ceil(e), n = Math.floor(t);
        return Math.floor(Math.random() * (n - r + 1) + r);
    }(o, i));
};

var _P = {};

Object.defineProperty(_P, "__esModule", {
    value: !0
}), _P.default = function() {
    return !0;
};

var EP = {};

Object.defineProperty(EP, "__esModule", {
    value: !0
}), EP.default = function(e) {
    return function() {
        return e;
    };
};

var bP = {}, wP = g && g.__values || function(e) {
    var t = "function" == typeof Symbol && Symbol.iterator, r = t && e[t], n = 0;
    if (r) {
        return r.call(e);
    }
    if (e && "number" == typeof e.length) {
        return {
            next: function() {
                return e && n >= e.length && (e = void 0), {
                    value: e && e[n++],
                    done: !e
                };
            }
        };
    }
    throw new TypeError(t ? "Object is not iterable." : "Symbol.iterator is not defined.");
}, DP = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(bP, "__esModule", {
    value: !0
});

var SP = DP(W_), AP = DP(dM), OP = DP(S_), CP = DP(q_), xP = DP(lE()), FP = DP(f_), MP = DP(c_), PP = DP(qO);

function IP(e, t) {
    var r = [], n = [];
    if ((0, OP.default)(e) || !(0, FP.default)(e)) {
        return r;
    }
    for (var o = e.length, i = function(o) {
        var i = e[o], a = t(i);
        -1 === n.findIndex(function(e) {
            return (0, AP.default)(a, e);
        }) && (r.push(i), n.push(a));
    }, a = 0; a < o; a++) {
        i(a);
    }
    return r;
}

bP.default = function(e, t) {
    void 0 === t && (t = SP.default);
    var r = t;
    (0, OP.default)(r) && (r = SP.default);
    var n, o = typeof r;
    return "function" === o ? IP(e, r) : "number" === o ? IP(e, (n = r, function(e) {
        return e[n];
    })) : "string" === o ? IP(e, function(e) {
        return function(t) {
            return (0, xP.default)(t, e);
        };
    }(r)) : (0, CP.default)(r) ? IP(e, function(e) {
        return function(t) {
            var r = e[0];
            return e[1] === (0, xP.default)(t, r);
        };
    }(r)) : IP(e, "object" === o ? function(e) {
        var t, r, n = (0, MP.default)(e), o = [];
        try {
            for (var i = wP(n), a = i.next(); !a.done; a = i.next()) {
                var s = a.value;
                o.push([ s, e[s] ]);
            }
        } catch (e) {
            t = {
                error: e
            };
        } finally {
            try {
                a && !a.done && (r = i.return) && r.call(i);
            } finally {
                if (t) {
                    throw t.error;
                }
            }
        }
        return function(e) {
            for (var t = o.length, r = 0; r < t; r++) {
                var n = o[r], i = e[n[0]];
                if (!(0, PP.default)(i, n[1])) {
                    return !1;
                }
            }
            return !0;
        };
    }(r) : SP.default);
};

var kP = {}, RP = {}, TP = {};

Object.defineProperty(TP, "__esModule", {
    value: !0
});

var jP = "\\ud800-\\udfff", LP = "\\u2700-\\u27bf", NP = "a-z\\xdf-\\xf6\\xf8-\\xff", BP = "A-Z\\xc0-\\xd6\\xd8-\\xde", UP = "\\xac\\xb1\\xd7\\xf7\\x00-\\x2f\\x3a-\\x40\\x5b-\\x60\\x7b-\\xbf\\u2000-\\u206f \\t\\x0b\\f\\xa0\\ufeff\\n\\r\\u2028\\u2029\\u1680\\u180e\\u2000\\u2001\\u2002\\u2003\\u2004\\u2005\\u2006\\u2007\\u2008\\u2009\\u200a\\u202f\\u205f\\u3000", zP = "['’]", HP = "[".concat(UP, "]"), $P = "[".concat("\\u0300-\\u036f\\ufe20-\\ufe2f\\u20d0-\\u20ff\\u1ab0-\\u1aff\\u1dc0-\\u1dff", "]"), GP = "[".concat(LP, "]"), WP = "[".concat(NP, "]"), VP = "[^".concat(jP).concat(UP + "\\d" + LP + NP + BP, "]"), KP = "(?:".concat($P, "|").concat("\\ud83c[\\udffb-\\udfff]", ")"), qP = "[^".concat(jP, "]"), JP = "(?:\\ud83c[\\udde6-\\uddff]){2}", XP = "[\\ud800-\\udbff][\\udc00-\\udfff]", ZP = "[".concat(BP, "]"), YP = "(?:".concat(WP, "|").concat(VP, ")"), QP = "(?:".concat(ZP, "|").concat(VP, ")"), eI = "(?:".concat(zP, "(?:d|ll|m|re|s|t|ve))?"), tI = "(?:".concat(zP, "(?:D|LL|M|RE|S|T|VE))?"), rI = "".concat(KP, "?"), nI = "[".concat("\\ufe0e\\ufe0f", "]?"), oI = nI + rI + "(?:".concat("\\u200d", "(?:").concat([ qP, JP, XP ].join("|"), ")").concat(nI + rI, ")*"), iI = "(?:".concat([ GP, JP, XP ].join("|"), ")").concat(oI), aI = RegExp([ "".concat(ZP, "?").concat(WP, "+").concat(eI, "(?=").concat([ HP, ZP, "$" ].join("|"), ")"), "".concat(QP, "+").concat(tI, "(?=").concat([ HP, ZP + YP, "$" ].join("|"), ")"), "".concat(ZP, "?").concat(YP, "+").concat(eI), "".concat(ZP, "+").concat(tI), "\\d*(?:1ST|2ND|3RD|(?![123])\\dTH)(?=\\b|[a-z_])", "\\d*(?:1st|2nd|3rd|(?![123])\\dth)(?=\\b|[A-Z_])", "".concat("\\d", "+"), iI ].join("|"), "g");

TP.default = function(e) {
    return e.match(aI);
};

var sI = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(RP, "__esModule", {
    value: !0
});

var uI = sI(TP);

RP.default = function(e, t) {
    return void 0 === e && (e = ""), void 0 === t && (t = void 0), void 0 === t ? (0, 
    uI.default)(e) || [] : e.match(t) || [];
};

var lI = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(kP, "__esModule", {
    value: !0
});

var cI = lI(S_), fI = lI(YC), dI = lI(RP);

kP.default = function(e) {
    if (void 0 === e && (e = ""), (0, cI.default)(e)) {
        return e;
    }
    var t = e;
    return "string" != typeof e && (t = (0, fI.default)(e)), (0, dI.default)(t.replace(/['\u2019]/g, "")).reduce(function(e, t, r) {
        return e + (r ? "_" : "") + t.toLowerCase();
    }, "");
};

var pI = {}, hI = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(pI, "__esModule", {
    value: !0
});

var vI = hI(S_), gI = hI(YC), mI = hI(RP);

pI.default = function(e) {
    if (void 0 === e && (e = ""), (0, vI.default)(e)) {
        return e;
    }
    var t = e;
    return "string" != typeof e && (t = (0, gI.default)(e)), (0, mI.default)(t.replace(/['\u2019]/g, "")).reduce(function(e, t, r) {
        return e + (r ? "-" : "") + t.toLowerCase();
    }, "");
};

var yI = {}, _I = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(yI, "__esModule", {
    value: !0
});

var EI = _I(S_), bI = _I(YC), wI = _I(RP);

yI.default = function(e) {
    if (void 0 === e && (e = ""), (0, EI.default)(e)) {
        return e;
    }
    var t = e;
    return "string" != typeof e && (t = (0, bI.default)(e)), (0, wI.default)(t.replace(/['\u2019]/g, "")).reduce(function(e, t, r) {
        return e + (r ? " " : "") + t.toLowerCase();
    }, "");
};

var DI = {}, SI = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(DI, "__esModule", {
    value: !0
});

var AI = SI(S_), OI = SI(YC), CI = SI(RP), xI = SI(aM), FI = SI(KF);

DI.default = function(e) {
    if (void 0 === e && (e = ""), (0, AI.default)(e)) {
        return e;
    }
    var t = e;
    return "string" != typeof e && (t = (0, OI.default)(e)), (0, CI.default)(t.replace(/['\u2019]/g, "")).reduce(function(e, t, r) {
        var n = (0, FI.default)(t);
        return e + (0 === r ? n : (0, xI.default)(n));
    }, "");
};

var MI = {}, PI = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(MI, "__esModule", {
    value: !0
});

var II = PI(f_), kI = fE();

MI.default = function(e) {
    if (null == e) {
        return 0;
    }
    if ((0, II.default)(e)) {
        return e.length;
    }
    var t = (0, kI.tagName)(e);
    return "[object Map]" === t || "[object Set]" === t ? e.size : Object.keys(e).length;
};

var RI = {};

Object.defineProperty(RI, "__esModule", {
    value: !0
}), RI.default = function(e, t, r) {
    void 0 === t && (t = 1);
    var n = null == e ? 0 : e.length;
    if (!n) {
        return [];
    }
    var o = t;
    if (r || void 0 === o ? o = 1 : o || (o = 0), 0 === o) {
        return [];
    }
    var i = n - o >= 0 ? n - o : 0;
    return e.slice(i);
};

var TI = {};

Object.defineProperty(TI, "__esModule", {
    value: !0
}), TI.default = function(e, t, r) {
    var n = "".concat(e);
    return null == t || null == r ? n : n.replace(t, r);
};

var jI = {}, LI = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, NI = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
}, BI = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(jI, "__esModule", {
    value: !0
});

var UI = BI(W_), zI = BI(Ab);

jI.default = function(e, t) {
    var r = {};
    if (!e) {
        return {};
    }
    var n = (0, zI.default)(e), o = (0, zI.default)(Object.getPrototypeOf(e));
    return n.push.apply(n, NI([], LI(o), !1)), n.forEach(function(n) {
        (t ? t(e[n], n, e) : (0, UI.default)(e[n])) && (r[n] = e[n]);
    }), r;
};

var HI = {}, $I = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(HI, "__esModule", {
    value: !0
});

var GI = $I(aM), WI = $I(YC);

HI.default = function(e) {
    return (0, GI.default)((0, WI.default)(e).toLowerCase());
};

var VI = {}, KI = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(VI, "__esModule", {
    value: !0
});

var qI = KI(jE);

VI.default = function(e, t) {
    var r = "function" == typeof t ? t : void 0;
    return (0, qI.default)(e, 5, r, void 0, void 0, void 0);
};

var JI = {}, XI = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(JI, "__esModule", {
    value: !0
});

var ZI = XI(YC), YI = {
    "&": "&amp;",
    "<": "&lt;",
    ">": "&gt;",
    '"': "&quot;",
    "'": "&#39;"
}, QI = /[&<>"']/g;

JI.default = function(e) {
    return (0, ZI.default)(e).replace(QI, function(e) {
        return YI[e];
    });
};

var ek = {}, tk = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(ek, "__esModule", {
    value: !0
});

var rk = tk(YC), nk = {
    "&amp;": "&",
    "&lt;": "<",
    "&gt;": ">",
    "&quot;": '"',
    "&#39;": "'"
}, ok = /&(?:amp|lt|gt|quot|#(0+)?39);/g;

ek.default = function(e) {
    return (0, rk.default)(e).replace(ok, function(e) {
        var t;
        return null !== (t = nk[e]) && void 0 !== t ? t : "'";
    });
};

var ik = {}, ak = {};

Object.defineProperty(ak, "__esModule", {
    value: !0
});

ak.default = /<%=([\s\S]+?)%>/g;

var sk = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(ik, "__esModule", {
    value: !0
});

var uk = sk(ak), lk = sk(JI), ck = {
    escape: /<%-([\s\S]+?)%>/g,
    evaluate: /<%([\s\S]+?)%>/g,
    interpolate: uk.default,
    variable: "",
    imports: {
        _: {
            escape: lk.default
        }
    }
};

ik.default = ck;

var fk = {}, dk = g && g.__assign || function() {
    return dk = Object.assign || function(e) {
        for (var t, r = 1, n = arguments.length; r < n; r++) {
            for (var o in t = arguments[r]) {
                Object.prototype.hasOwnProperty.call(t, o) && (e[o] = t[o]);
            }
        }
        return e;
    }, dk.apply(this, arguments);
}, pk = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(fk, "__esModule", {
    value: !0
});

var hk = pk(ik), vk = pk(YC), gk = pk(ak), mk = pk(IC), yk = "Invalid `variable` settings for template function", _k = /\b__p \+= '';/g, Ek = /\b(__p \+=) '' \+/g, bk = /(__e\(.*?\)|\b__t\)) \+\n'';/g, wk = /[()=,{}[\]/\s]/, Dk = /\$\{([^\\}]*(?:\\.[^\\}]*)*)\}/g, Sk = /($^)/, Ak = /['\n\r\u2028\u2029\\]/g, Ok = {
    "\\": "\\",
    "'": "'",
    "\n": "n",
    "\r": "r",
    "\u2028": "u2028",
    "\u2029": "u2029"
};

function Ck(e) {
    return "\\".concat(Ok[e]);
}

var xk = Object.prototype.hasOwnProperty;

fk.default = function(e, t, r) {
    var n = hk.default.imports._.templateSettings || hk.default, o = t;
    r && r[o] === e && (o = void 0);
    var i, a = (0, vk.default)(e), s = (0, mk.default)({}, n, o), u = s.imports, l = xk.call(s, "sourceURL") ? "//# sourceURL=".concat("".concat(s.sourceURL).replace(/\s/g, " "), "\n") : "", c = function(e) {
        var t = e.templateString, r = e.mergedOptions, n = r.variable || "obj";
        if (wk.test(n)) {
            throw new Error(yk);
        }
        var o, i, a = r.interpolate || Sk, s = 0, u = "__p += '", l = RegExp("".concat((r.escape || Sk).source, "|").concat(a.source, "|").concat((a === gk.default ? Dk : Sk).source, "|").concat((r.evaluate || Sk).source, "|$"), "g");
        return t.replace(l, function(e, r, n, a, l, c) {
            var f = n || a;
            return u += t.slice(s, c).replace(Ak, Ck), r && (i = !0, u += "' +\n__e(".concat(r, ") +\n'")), 
            l && (o = !0, u += "';\n".concat(l, ";\n__p += '")), f && (u += "' +\n((__t = (".concat(f, ")) == null ? '' : __t) +\n'")), 
            s = c + e.length, e;
        }), u += "';\n", u = (o ? u.replace(_k, "") : u).replace(Ek, "$1").replace(bk, "$1;"), 
        u = "function (".concat(n, " = {}) {\nlet __t, __p = '' , _ = ").concat(n, "['_']\n  ").concat(i ? ", __e = _.escape" : "").concat(o ? ", __join = Array.prototype.join;\nfunction print() { __p += __join.call(arguments, '') }\n" : "", ";with(").concat(n, "){\n").concat(u, "}\nreturn __p\n}"), 
        u;
    }({
        templateString: a,
        mergedOptions: s
    });
    try {
        var f = Function("".concat(l, "return ").concat(c))();
        i = function(e) {
            void 0 === e && (e = {});
            var t = dk(dk({}, u), e);
            return null == f ? void 0 : f.call(this, t);
        };
    } catch (e) {
        i = e;
    }
    if (i.source = c, i instanceof Error) {
        throw i;
    }
    return i;
};

var Fk = {}, Mk = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, Pk = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
}, Ik = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Fk, "__esModule", {
    value: !0
});

var kk = eD, Rk = Ik(PS);

Fk.default = function(e) {
    for (var t = [], r = 1; r < arguments.length; r++) {
        t[r - 1] = arguments[r];
    }
    var n, o = t, i = t.length, a = t[i - 1];
    return Array.isArray(a) || (o = t.slice(0, i - 1), n = t[i - 1]), (o = o.filter(function(e) {
        return (0, Rk.default)(e);
    })).length && (o = o.reduce(function(e, t) {
        return Pk(Pk([], Mk(e), !1), Mk(t), !1);
    })), (0, kk.baseDifference)(e, o, n, void 0);
};

var Tk = {};

Object.defineProperty(Tk, "__esModule", {
    value: !0
}), Tk.default = function(e, t) {
    var r = {};
    if (null == e) {
        return r;
    }
    var n = t;
    for (var o in null == n ? n = function() {
        return !0;
    } : "function" != typeof n && (n = function() {
        return !1;
    }), e) {
        n(e[o], o) || (r[o] = e[o]);
    }
    for (var i = Object.getOwnPropertySymbols(e), a = Object.getPrototypeOf(e), s = function(e) {
        return a.propertyIsEnumerable.call(a, e);
    }; a; ) {
        i = i.concat(Object.getOwnPropertySymbols(a).filter(s)), a = Object.getPrototypeOf(a);
    }
    return i.forEach(function(t) {
        n(e[t], t) || (r[t] = e[t]);
    }), r;
};

var jk = {}, Lk = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, Nk = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
};

Object.defineProperty(jk, "__esModule", {
    value: !0
});

var Bk = Symbol("placeholder");

function Uk(e, t) {
    void 0 === t && (t = e.length);
    var r = parseInt(t, 10);
    r = Number.isNaN(r) ? 0 : Math.floor(r);
    var n = Uk.placeholder;
    function o() {
        for (var t = [], o = 0; o < arguments.length; o++) {
            t[o] = arguments[o];
        }
        var i = Nk([], Lk(t), !1), a = zk(i, n), s = i.length - a.length;
        return this && Object.setPrototypeOf(this, e.prototype), s < r ? Hk(e, r - s, i, a, n) : e.apply(this, i);
    }
    return o.placeholder = n, o;
}

function zk(e, t) {
    var r = [];
    return e.forEach(function(e, n) {
        e === t && r.push(n);
    }), r;
}

function Hk(e, t, r, n, o) {
    function i() {
        for (var i = [], a = 0; a < arguments.length; a++) {
            i[a] = arguments[a];
        }
        var s = Nk([], Lk(i), !1), u = zk(s, o).length, l = function(e, t, r) {
            for (var n = -1, o = -1, i = Math.max(r.length - t.length, 0), a = new Array(e.length + i); ++n < e.length; ) {
                a[n] = e[n];
            }
            for (;++o < t.length && o < r.length; ) {
                a[t[o]] = r[o];
            }
            for (;i--; ) {
                a[n++] = r[o++];
            }
            return a;
        }(r, n, s), c = zk(l, o), f = s.length - u;
        return f < t ? Hk(e, t - f, l, c, o) : e.apply(this, l);
    }
    return i.placeholder = o, i;
}

Uk.placeholder = Bk, jk.default = Uk;

var $k = {};

Object.defineProperty($k, "__esModule", {
    value: !0
}), $k.default = function(e, t) {
    return (null == e ? void 0 : e.length) ? e.map(function(e) {
        return "function" == typeof t ? t(e) : e[t];
    }).reduce(function(e, t) {
        return e + t;
    }) / e.length : NaN;
};

var Gk = {}, Wk = g && g.__values || function(e) {
    var t = "function" == typeof Symbol && Symbol.iterator, r = t && e[t], n = 0;
    if (r) {
        return r.call(e);
    }
    if (e && "number" == typeof e.length) {
        return {
            next: function() {
                return e && n >= e.length && (e = void 0), {
                    value: e && e[n++],
                    done: !e
                };
            }
        };
    }
    throw new TypeError(t ? "Object is not iterable." : "Symbol.iterator is not defined.");
};

Object.defineProperty(Gk, "__esModule", {
    value: !0
});

var Vk = uE();

Gk.default = function(e, t, r) {
    var n, o;
    void 0 === t && (t = []);
    var i, a = (0, Vk.getObjValidPathFromGeneralPath)(e, t), s = e;
    try {
        for (var u = Wk(a), l = u.next(); !l.done; l = u.next()) {
            if ("function" == typeof (i = s[l.value]) && (i = i.call(s)), null == i) {
                break;
            }
            s = i;
        }
    } catch (e) {
        n = {
            error: e
        };
    } finally {
        try {
            l && !l.done && (o = u.return) && o.call(u);
        } finally {
            if (n) {
                throw n.error;
            }
        }
    }
    return null == i ? "function" == typeof r ? r.call(s) : r : i;
};

var Kk = {}, qk = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Kk, "__esModule", {
    value: !0
});

var Jk = qk(f_);

Kk.default = function() {
    for (var e = [], t = 0; t < arguments.length; t++) {
        e[t] = arguments[t];
    }
    if (!(0, Jk.default)(e) && "string" != typeof e) {
        return [];
    }
    for (var r = [], n = function(e) {
        var t = 0;
        return e.forEach(function(e) {
            (0, Jk.default)(e) && "string" != typeof e && e.length > t && (t = e.length);
        }), t;
    }(e), o = function(t) {
        var n = [];
        e.forEach(function(e) {
            (0, Jk.default)(e) && "string" != typeof e && n.push(e[t]);
        }), r.push(n);
    }, i = 0; i < n; i++) {
        o(i);
    }
    return r;
};

var Xk = {}, Zk = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Xk, "__esModule", {
    value: !0
});

var Yk = eD, Qk = OS, eR = Zk(PS);

Xk.default = function(e) {
    for (var t, r = [], n = 1; n < arguments.length; n++) {
        r[n - 1] = arguments[n];
    }
    var o = r, i = r.length, a = r[i - 1];
    return "function" == typeof a && (t = a, o = r.slice(0, i - 1)), (0, Yk.baseDifference)(e, (0, 
    Qk.baseFlatten)(o, 1, eR.default, !0, void 0), void 0, t);
};

var tR = {}, rR = g && g.__values || function(e) {
    var t = "function" == typeof Symbol && Symbol.iterator, r = t && e[t], n = 0;
    if (r) {
        return r.call(e);
    }
    if (e && "number" == typeof e.length) {
        return {
            next: function() {
                return e && n >= e.length && (e = void 0), {
                    value: e && e[n++],
                    done: !e
                };
            }
        };
    }
    throw new TypeError(t ? "Object is not iterable." : "Symbol.iterator is not defined.");
};

Object.defineProperty(tR, "__esModule", {
    value: !0
});

tR.default = function(e) {
    var t, r;
    if (!Array.isArray(e)) {
        return [];
    }
    for (var n = [], o = function(e) {
        var t, r, n = 0;
        try {
            for (var o = rR(e), i = o.next(); !i.done; i = o.next()) {
                var a = i.value;
                Array.isArray(a) && a.length > n && (n = a.length);
            }
        } catch (e) {
            t = {
                error: e
            };
        } finally {
            try {
                i && !i.done && (r = o.return) && r.call(o);
            } finally {
                if (t) {
                    throw t.error;
                }
            }
        }
        return n;
    }(e), i = 0; i < o; i++) {
        var a = [];
        try {
            for (var s = (t = void 0, rR(e)), u = s.next(); !u.done; u = s.next()) {
                var l = u.value;
                Array.isArray(l) && a.push(l[i]);
            }
        } catch (e) {
            t = {
                error: e
            };
        } finally {
            try {
                u && !u.done && (r = s.return) && r.call(s);
            } finally {
                if (t) {
                    throw t.error;
                }
            }
        }
        n.push(a);
    }
    return n;
};

var nR = {};

Object.defineProperty(nR, "__esModule", {
    value: !0
}), nR.default = function(e, t) {
    try {
        return Number.parseInt(e, t);
    } catch (e) {
        return Number.NaN;
    }
};

var oR = {}, iR = g && g.__values || function(e) {
    var t = "function" == typeof Symbol && Symbol.iterator, r = t && e[t], n = 0;
    if (r) {
        return r.call(e);
    }
    if (e && "number" == typeof e.length) {
        return {
            next: function() {
                return e && n >= e.length && (e = void 0), {
                    value: e && e[n++],
                    done: !e
                };
            }
        };
    }
    throw new TypeError(t ? "Object is not iterable." : "Symbol.iterator is not defined.");
};

Object.defineProperty(oR, "__esModule", {
    value: !0
});

var aR = uE();

oR.default = function(e, t) {
    var r, n;
    if (null == e || null == t) {
        return !0;
    }
    var o = (0, aR.getObjValidPathFromGeneralPath)(e, t);
    if (!o.length) {
        return !0;
    }
    var i = e;
    try {
        for (var a = iR(o.splice(0, o.length - 1)), s = a.next(); !s.done; s = a.next()) {
            if (null == (i = i[s.value])) {
                break;
            }
        }
    } catch (e) {
        r = {
            error: e
        };
    } finally {
        try {
            s && !s.done && (n = a.return) && n.call(a);
        } finally {
            if (r) {
                throw r.error;
            }
        }
    }
    if (null == i) {
        return !0;
    }
    try {
        return delete i[o.pop()];
    } catch (e) {
        return !1;
    }
};

var sR = {};

Object.defineProperty(sR, "__esModule", {
    value: !0
});

var uR = Hy, lR = uE();

sR.default = function(e, t, r, n) {
    if (null == e) {
        return e;
    }
    var o = (0, lR.getObjValidPathFromGeneralPath)(e, t);
    if (0 === o.length) {
        return e;
    }
    for (var i = e, a = 0; a < o.length - 1; a++) {
        if ("object" != typeof i) {
            return e;
        }
        var s = o[a], u = i[s], l = "function" == typeof n ? n(u, s, i) : void 0;
        null == l && (l = null == u ? (0, uR.isIndex)(o[a + 1]) ? [] : {} : u), i[s] = l, 
        i = l;
    }
    if ("object" != typeof i) {
        return e;
    }
    var c = o[o.length - 1], f = "function" == typeof r ? r(i[c]) : void 0;
    return i[c] = f, e;
};

var cR = {}, fR = {}, dR = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, pR = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
};

Object.defineProperty(fR, "__esModule", {
    value: !0
}), fR.needDeeperCompare = void 0;

var hR = fE(), vR = Array.isArray;

function gR(e) {
    return "object" == typeof e && null !== e;
}

function mR(e, t) {
    return gR(e) && gR(t);
}

fR.needDeeperCompare = mR, fR.default = function e(t, r) {
    var n = function(e, t) {
        var r = Object.fromEntries, n = e, o = (0, hR.tagName)(e);
        vR(e) || "[object Set]" === o ? n = pR([], dR(e), !1) : "[object Map]" === o && (n = r(e.entries()));
        var i = t, a = (0, hR.tagName)(t);
        return "[object Set]" === a ? i = pR([], dR(t), !1) : "[object Map]" === a && (i = r(t.entries())), 
        {
            obj: n,
            source: i
        };
    }(t, r), o = n.obj, i = n.source, a = null == o ? [] : (0, hR.getObjectKeysWithProtoChain)(o), s = null == i ? [] : Object.keys(i);
    if (vR(o) && vR(i)) {
        for (var u = function(t, r) {
            var n = s[t], a = i[n], u = o.findIndex(function(t) {
                return gR(a) ? e(t, a) : t === a;
            });
            return -1 !== u ? (o.splice(u, 1), "continue") : {
                value: !1
            };
        }, l = 0, c = s.length; l < c; l++) {
            var f = u(l);
            if ("object" == typeof f) {
                return f.value;
            }
        }
        return !0;
    }
    for (var d = Number.isNaN, p = (l = 0, s.length); l < p; l++) {
        var h = s[l];
        if (!a.includes(h)) {
            return !1;
        }
        var v = o[h], g = i[h];
        if (mR(v, g)) {
            if (!e(v, g)) {
                return !1;
            }
        } else {
            if (d(g) && d(o)) {
                continue;
            }
            if (v !== g) {
                return !1;
            }
        }
    }
    return !0;
};

var yR = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(cR, "__esModule", {
    value: !0
});

var _R = yR(fR), ER = yR(Mw);

cR.default = function(e) {
    var t = (0, ER.default)(e);
    return function(e) {
        return (0, _R.default)(e, t);
    };
};

var bR = {}, wR = g && g.__createBinding || (Object.create ? function(e, t, r, n) {
    void 0 === n && (n = r);
    var o = Object.getOwnPropertyDescriptor(t, r);
    o && !("get" in o ? !t.__esModule : o.writable || o.configurable) || (o = {
        enumerable: !0,
        get: function() {
            return t[r];
        }
    }), Object.defineProperty(e, n, o);
} : function(e, t, r, n) {
    void 0 === n && (n = r), e[n] = t[r];
}), DR = g && g.__setModuleDefault || (Object.create ? function(e, t) {
    Object.defineProperty(e, "default", {
        enumerable: !0,
        value: t
    });
} : function(e, t) {
    e.default = t;
}), SR = g && g.__importStar || function(e) {
    if (e && e.__esModule) {
        return e;
    }
    var t = {};
    if (null != e) {
        for (var r in e) {
            "default" !== r && Object.prototype.hasOwnProperty.call(e, r) && wR(t, e, r);
        }
    }
    return DR(t, e), t;
}, AR = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(bR, "__esModule", {
    value: !0
});

var OR = AR(lE()), CR = SR(fR), xR = uE(), FR = AR(Mw), MR = fE();

bR.default = function(e, t) {
    var r = (0, FR.default)(t);
    return function(t) {
        var n = (0, xR.getObjValidPathFromGeneralPath)(t, e), o = n.slice(0, n.length - 1), i = o.length ? (0, 
        OR.default)(t, o) : t, a = n[n.length - 1] || "";
        if (null == i) {
            return !1;
        }
        if ("[object Object]" === (0, MR.tagName)(i) && !(a in i)) {
            return !1;
        }
        i = i[a];
        var s = typeof r;
        return (0, CR.needDeeperCompare)(i, r) || "function" === s ? (0, CR.default)(i, r) : i === r;
    };
};

var PR = {}, IR = g && g.__values || function(e) {
    var t = "function" == typeof Symbol && Symbol.iterator, r = t && e[t], n = 0;
    if (r) {
        return r.call(e);
    }
    if (e && "number" == typeof e.length) {
        return {
            next: function() {
                return e && n >= e.length && (e = void 0), {
                    value: e && e[n++],
                    done: !e
                };
            }
        };
    }
    throw new TypeError(t ? "Object is not iterable." : "Symbol.iterator is not defined.");
}, kR = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(PR, "__esModule", {
    value: !0
});

var RR = fE(), TR = kR(cR), jR = kR(lE()), LR = kR(bR), NR = kR(c_), BR = kR(W_);

PR.default = function(e, t) {
    var r, n, o = Array.isArray(e) ? e : (0, NR.default)(e), i = typeof t, a = null == t ? BR.default : function() {
        return !0;
    };
    "function" === i ? a = t : "string" === i ? a = function(e) {
        return (0, jR.default)(e, t);
    } : "[object Object]" === (0, RR.tagName)(t) ? a = (0, TR.default)(t) : Array.isArray(t) && (a = (0, 
    LR.default)(t[0], t[1]));
    try {
        for (var s = IR(o), u = s.next(); !u.done; u = s.next()) {
            if (!a(u.value)) {
                return !1;
            }
        }
    } catch (e) {
        r = {
            error: e
        };
    } finally {
        try {
            u && !u.done && (n = s.return) && n.call(s);
        } finally {
            if (r) {
                throw r.error;
            }
        }
    }
    return !0;
};

var UR = {}, zR = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, HR = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
}, $R = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(UR, "__esModule", {
    value: !0
});

var GR = $R(xA);

UR.default = function() {
    for (var e = [], t = 0; t < arguments.length; t++) {
        e[t] = arguments[t];
    }
    var r = (0, GR.default)(e);
    return function() {
        for (var e = [], t = 0; t < arguments.length; t++) {
            e[t] = arguments[t];
        }
        var n = HR([], zR(e), !1);
        return r.forEach(function(e) {
            n = [ e.apply(void 0, HR([], zR(n), !1)) ];
        }), n[0];
    };
};

var WR = {}, VR = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, KR = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
}, qR = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(WR, "__esModule", {
    value: !0
});

var JR = qR(fM);

WR.default = function() {
    for (var e = [], t = 0; t < arguments.length; t++) {
        e[t] = arguments[t];
    }
    return JR.default.apply(void 0, KR([], VR(e), !1));
};

var XR = {}, ZR = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(XR, "__esModule", {
    value: !0
});

var YR = ZR(UO), QR = ZR(RF);

XR.default = function(e, t) {
    return void 0 === t && (t = 1), (0, YR.default)(e) ? [] : (0, QR.default)(e, 0, t < 0 ? 0 : t);
};

var eT = {}, tT = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, rT = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
}, nT = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(eT, "__esModule", {
    value: !0
});

var oT = fE(), iT = nT(c_);

eT.default = function(e, t) {
    var r = (0, oT.tagName)(e);
    if ("[object Map]" === r) {
        return Array.from(e.entries());
    }
    var n = [], o = [];
    return "[object Set]" === r ? n = (o = rT([], tT(e), !1)).map(function(e, t) {
        return t + 1;
    }) : (n = t ? (0, oT.getObjectKeysWithProtoChain)(e) : (0, iT.default)(e), o = n.map(function(t) {
        return e[t];
    })), n.map(function(e, t) {
        return [ e, o[t] ];
    });
};

var aT = {};

Object.defineProperty(aT, "__esModule", {
    value: !0
}), aT.default = function(e) {
    var t = {};
    return Array.isArray(e) ? (e.forEach(function(e) {
        null != e && (t[e[0]] = e[1]);
    }), t) : t;
};

var sT = {}, uT = {}, lT = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(uT, "__esModule", {
    value: !0
});

var cT = lT($y);

uT.default = function(e) {
    return "[object RegExp]" === (0, cT.default)(e);
};

var fT = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(sT, "__esModule", {
    value: !0
});

var dT = fT(S_), pT = fT(YC), hT = fT(uT), vT = fT(TA), gT = fT(GS), mT = fT(qA), yT = {
    length: 30,
    omission: "..."
};

function _T(e) {
    var t = {};
    return function(e, t) {
        (0, mT.default)(e, "omission") ? t.omission = e.omission : t.omission = yT.omission, 
        (0, mT.default)(e, "length") ? t.length = e.length : t.length = yT.length, (0, mT.default)(e, "separator") && (t.separator = e.separator);
    }(e, t), null === t.omission && (t.omission = "null"), void 0 === t.omission && (t.omission = "undefined"), 
    void 0 === t.length && (t.length = yT.length), t.length = (0, gT.default)(t.length), 
    t.length < 0 && (t.length = 0), t;
}

sT.default = function(e, t) {
    void 0 === e && (e = "");
    var r = (0, dT.default)(t) ? yT : t;
    r = _T(r);
    var n = (0, pT.default)(e);
    if (n.length <= r.length) {
        return e;
    }
    var o = n.substring(0, r.length), i = function(e, t) {
        var r = e.length;
        if (!(0, dT.default)(t.separator)) {
            var n = (0, vT.default)(t.separator) ? RegExp(t.separator) : t.separator;
            (0, hT.default)(n) || (n = RegExp((0, pT.default)(t.separator))), n.global || (n = RegExp(n.source, "g"));
            for (var o = e.matchAll(n), i = o.next(); !i.done; ) {
                r = i.value.index, i = o.next();
            }
        }
        return r;
    }(o, r);
    if (i + r.omission.length > o.length) {
        var a = 2 * o.length - r.omission.length - i;
        return o.substring(0, a) + r.omission;
    }
    return o.substring(0, i) + r.omission;
};

var ET = {};

Object.defineProperty(ET, "__esModule", {
    value: !0
});

var bT = IC, wT = (0, zy.createAssignFunction)(function(e, t, r, n) {
    (0, bT.baseMerge)(e, t, r, n);
});

ET.default = wT;

var DT = {}, ST = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(DT, "__esModule", {
    value: !0
});

var AT = ST(aT), OT = ST(XR), CT = ST(Kk);

DT.default = function(e, t) {
    return void 0 === e && (e = []), void 0 === t && (t = []), (0, AT.default)((0, CT.default)(e, (0, 
    OT.default)(t, e.length)));
};

var xT = {}, FT = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, MT = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
}, PT = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(xT, "__esModule", {
    value: !0
});

var IT = PT(lE()), kT = uE();

xT.default = function(e, t) {
    for (var r, n = [], o = 2; o < arguments.length; o++) {
        n[o - 2] = arguments[o];
    }
    var i = (0, kT.getObjValidPathFromGeneralPath)(e, t), a = i.length, s = i[a - 1], u = a > 1 ? (0, 
    IT.default)(e, i.slice(0, a - 1)) : e;
    return null == u || null === (r = u[s]) || void 0 === r ? void 0 : r.call.apply(r, MT([ u ], FT(n), !1));
};

var RT = {}, TT = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(RT, "__esModule", {
    value: !0
});

var jT = TT(S_), LT = TT(YC), NT = TT(RP), BT = TT(ZF);

RT.default = function(e) {
    if (void 0 === e && (e = ""), (0, jT.default)(e)) {
        return e;
    }
    var t = e;
    return "string" != typeof e && (t = (0, LT.default)(e)), (0, NT.default)(t.replace(/['\u2019]/g, "")).reduce(function(e, t, r) {
        var n = (0, BT.default)(t);
        return e + (0 === r ? n : " ".concat(n));
    }, "");
};

var UT = {}, zT = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(UT, "__esModule", {
    value: !0
});

var HT = fE(), $T = zT(cR), GT = zT(lE()), WT = zT(bR), VT = zT(c_), KT = zT(W_);

UT.default = function(e, t) {
    var r = Array.isArray(e) ? e : (0, VT.default)(e), n = typeof t, o = null == t ? KT.default : function() {
        return !0;
    };
    "function" === n ? o = t : "string" === n ? o = function(e) {
        return (0, GT.default)(e, t);
    } : "[object Object]" === (0, HT.tagName)(t) ? o = (0, $T.default)(t) : Array.isArray(t) && (o = (0, 
    WT.default)(t[0], t[1]));
    for (var i = 0; i < r.length; i++) {
        if (o(r[i])) {
            return !0;
        }
    }
    return !1;
};

var qT = {}, JT = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(qT, "__esModule", {
    value: !0
});

var XT = zy, ZT = r_, YT = JT(c_), QT = (0, XT.createAssignFunction)(function(e, t, r, n) {
    (0, ZT.copyObject)(t, (0, YT.default)(t), e, n);
});

qT.default = QT;

var ej = {}, tj = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(ej, "__esModule", {
    value: !0
});

var rj = fE(), nj = tj(bR), oj = tj(cE()), ij = tj(c_), aj = tj(cR);

ej.default = function(e, t) {
    if (null != e) {
        var r = (0, ij.default)(e), n = function() {
            return !1;
        }, o = typeof t, i = (0, rj.tagName)(t);
        "function" === o ? n = t : Array.isArray(t) ? n = (0, nj.default)(t[0], t[1]) : "[object Object]" === i ? n = (0, 
        aj.default)(t) : "string" === o && (n = (0, oj.default)(t));
        for (var a = 0, s = r.length; a < s; a++) {
            var u = r[a];
            if (Boolean(n(e[u], u, e))) {
                return u;
            }
        }
    }
};

var sj = {}, uj = g && g.__values || function(e) {
    var t = "function" == typeof Symbol && Symbol.iterator, r = t && e[t], n = 0;
    if (r) {
        return r.call(e);
    }
    if (e && "number" == typeof e.length) {
        return {
            next: function() {
                return e && n >= e.length && (e = void 0), {
                    value: e && e[n++],
                    done: !e
                };
            }
        };
    }
    throw new TypeError(t ? "Object is not iterable." : "Symbol.iterator is not defined.");
}, lj = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(sj, "__esModule", {
    value: !0
});

var cj = lj(UO), fj = lj(mC), dj = lj(cE());

sj.default = function(e, t) {
    var r, n;
    if ((0, cj.default)(e)) {
        return 0;
    }
    var o = 0, i = (0, fj.default)(t) ? t : (0, dj.default)(t);
    try {
        for (var a = uj(e), s = a.next(); !s.done; s = a.next()) {
            var u = i(s.value);
            void 0 !== u && (o += u);
        }
    } catch (e) {
        r = {
            error: e
        };
    } finally {
        try {
            s && !s.done && (n = a.return) && n.call(a);
        } finally {
            if (r) {
                throw r.error;
            }
        }
    }
    return o;
};

var pj = {}, hj = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(pj, "__esModule", {
    value: !0
});

var vj = hj(bP), gj = hj(PS), mj = hj(OC), yj = hj(xA), _j = hj(iM);

pj.default = function() {
    for (var e = [], t = 0; t < arguments.length; t++) {
        e[t] = arguments[t];
    }
    var r = (0, mj.default)(e);
    return (0, gj.default)(r) ? (0, _j.default)((0, yj.default)(e)) : (e.pop(), (0, 
    vj.default)((0, yj.default)(e), r));
};

var Ej = {}, bj = g && g.__values || function(e) {
    var t = "function" == typeof Symbol && Symbol.iterator, r = t && e[t], n = 0;
    if (r) {
        return r.call(e);
    }
    if (e && "number" == typeof e.length) {
        return {
            next: function() {
                return e && n >= e.length && (e = void 0), {
                    value: e && e[n++],
                    done: !e
                };
            }
        };
    }
    throw new TypeError(t ? "Object is not iterable." : "Symbol.iterator is not defined.");
}, wj = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Ej, "__esModule", {
    value: !0
});

var Dj = wj(UO);

Ej.default = function(e, t) {
    var r, n;
    if ((0, Dj.default)(e)) {
        return [];
    }
    var o = [], i = "function" == typeof t ? t : void 0, a = function(e) {
        if (o.some(function(t) {
            return i(t, e);
        })) {
            return "continue";
        }
        o.push(e);
    };
    try {
        for (var s = bj(e), u = s.next(); !u.done; u = s.next()) {
            a(u.value);
        }
    } catch (e) {
        r = {
            error: e
        };
    } finally {
        try {
            u && !u.done && (n = s.return) && n.call(s);
        } finally {
            if (r) {
                throw r.error;
            }
        }
    }
    return o;
};

var Sj = {}, Aj = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, Oj = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
}, Cj = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Sj, "__esModule", {
    value: !0
});

var xj = Hy, Fj = Cj(z_);

function Mj(e, t, r) {
    return -1 !== e.findIndex(function(e) {
        return r(t, e);
    });
}

var Pj = function(e, t) {
    return e === t || (0, Fj.default)(e) && (0, Fj.default)(t);
};

Sj.default = function() {
    for (var e = [], t = 0; t < arguments.length; t++) {
        e[t] = arguments[t];
    }
    var r = e.length, n = e[r - 1], o = e, i = Pj;
    "function" == typeof n && r > 1 && (i = n, o = e.slice(0, r - 1));
    var a = [], s = Array.isArray;
    if (-1 !== o.findIndex(function(e) {
        return !s(e) && !(0, xj.isArguments)(e);
    })) {
        return a;
    }
    o = o.map(function(e) {
        return Oj([], Aj(e), !1);
    }), i === Pj && (o = o.map(function(e) {
        return e.map(function(e) {
            return 0 === e ? 0 : e;
        });
    }));
    var u = o[0] || [];
    return r = o.length, u.forEach(function(e) {
        if (!Mj(a, e, i)) {
            for (var t = 1; t < r; t++) {
                if (!Mj(o[t], e, i)) {
                    return;
                }
            }
            a.push(e);
        }
    }), a;
};

var Ij = {}, kj = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Ij, "__esModule", {
    value: !0
});

var Rj = kj(xA), Tj = kj(W_), jj = kj(cE()), Lj = kj(c_);

Ij.default = function(e, t) {
    var r = Tj.default, n = typeof t;
    "function" === n ? r = t : "string" === n && (r = (0, jj.default)(t));
    var o = (0, Lj.default)(e);
    return (0, Rj.default)(o.map(function(t) {
        return r(e[t], t, e);
    }));
};

var Nj = {}, Bj = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Nj, "__esModule", {
    value: !0
});

var Uj = Bj(c_), zj = fE();

Nj.default = function(e, t) {
    var r = (0, zj.getRealIterateeWithIdentityDefault)(t), n = (0, Uj.default)(e), o = {};
    return n.forEach(function(t) {
        var n = e[t];
        o[r(n, t, e)] = n;
    }), o;
};

var Hj = {}, $j = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Hj, "__esModule", {
    value: !0
});

var Gj = fE(), Wj = $j(c_);

Hj.default = function(e, t) {
    var r = (0, Gj.getRealIterateeWithIdentityDefault)(t), n = (0, Wj.default)(e), o = {};
    return n.forEach(function(t) {
        o[t] = r(e[t], t, e);
    }), o;
};

var Vj = {}, Kj = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Vj, "__esModule", {
    value: !0
});

var qj = Kj(BC), Jj = fE();

Vj.default = function(e, t) {
    return (0, qj.default)(e, (0, Jj.getRealIterateeWithIdentityDefault)(t));
};

var Xj = {}, Zj = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Xj, "__esModule", {
    value: !0
});

var Yj = Hy, Qj = Zj(ob), eL = Zj(f_), tL = Zj(qE);

Xj.default = function(e, t, r) {
    var n = t, o = r;
    if (r && "number" != typeof r && function(e, t, r) {
        if (!(0, tL.default)(r)) {
            return !1;
        }
        var n = typeof t;
        if ("number" === n ? (0, eL.default)(r) && (0, Yj.isIndex)(t, r.length) : "string" === n && t in r) {
            return (0, Qj.default)(r[t], e);
        }
        return !1;
    }(e, t, r) && (n = void 0, o = void 0), !(o = void 0 === o ? 4294967295 : o >>> 0)) {
        return [];
    }
    var i = e;
    return null == e && (i = ""), i.split(n, o);
};

var rL = {}, nL = g && g.__read || function(e, t) {
    var r = "function" == typeof Symbol && e[Symbol.iterator];
    if (!r) {
        return e;
    }
    var n, o, i = r.call(e), a = [];
    try {
        for (;(void 0 === t || t-- > 0) && !(n = i.next()).done; ) {
            a.push(n.value);
        }
    } catch (e) {
        o = {
            error: e
        };
    } finally {
        try {
            n && !n.done && (r = i.return) && r.call(i);
        } finally {
            if (o) {
                throw o.error;
            }
        }
    }
    return a;
}, oL = g && g.__spreadArray || function(e, t, r) {
    if (r || 2 === arguments.length) {
        for (var n, o = 0, i = t.length; o < i; o++) {
            !n && o in t || (n || (n = Array.prototype.slice.call(t, 0, o)), n[o] = t[o]);
        }
    }
    return e.concat(n || Array.prototype.slice.call(t));
}, iL = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(rL, "__esModule", {
    value: !0
});

var aL = iL(hO), sL = fE();

rL.default = function(e) {
    return null == e ? [] : "[object Map]" === (0, sL.tagName)(e) ? oL([], nL(e.keys()), !1).map(function(t) {
        return [ t, e.get(t) ];
    }) : e[Symbol.iterator] === Array.prototype[Symbol.iterator] ? (0, aL.default)(oL([], nL(e), !1)) : (0, 
    aL.default)(e);
};

var uL = {}, lL = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(uL, "__esModule", {
    value: !0
});

var cL = lL($y);

uL.default = function(e) {
    return "[object RegExp]" === (0, cL.default)(e);
};

var fL = {};

Object.defineProperty(fL, "__esModule", {
    value: !0
}), fL.default = function(e) {
    return Number.isSafeInteger(e);
};

var dL = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Uy, "__esModule", {
    value: !0
}), Uy.isObject = Uy.isNumber = Uy.isNull = Uy.isNil = Uy.isFunction = Uy.isFinite = Uy.isEqual = Uy.isEmpty = Uy.isDate = Uy.isBuffer = Uy.isBoolean = Uy.isArrayLikeObject = Uy.isArrayLike = Uy.isArray = Uy.invert = Uy.intersection = Uy.indexOf = Uy.includes = Uy.head = Uy.has = Uy.groupBy = Uy.forIn = Uy.forEach = Uy.floor = Uy.flattenDeep = Uy.flatten = Uy.findLastIndex = Uy.findIndex = Uy.find = Uy.filter = Uy.eq = Uy.each = Uy.endsWith = Uy.dropRight = Uy.drop = Uy.divide = Uy.difference = Uy.defaultTo = Uy.debounce = Uy.concat = Uy.compact = Uy.cloneDeep = Uy.clone = Uy.clamp = Uy.chunk = Uy.ceil = Uy.castArray = Uy.extend = Uy.assignIn = Uy.assign = void 0, 
Uy.uniqueId = Uy.union = Uy.upperFirst = Uy.uniq = Uy.trimStart = Uy.trimEnd = Uy.trim = Uy.toUpper = Uy.toString = Uy.toNumber = Uy.toLower = Uy.throttle = Uy.sum = Uy.startsWith = Uy.sortBy = Uy.slice = Uy.set = Uy.round = Uy.reverse = Uy.remove = Uy.reduce = Uy.range = Uy.pullAt = Uy.pull = Uy.pick = Uy.padStart = Uy.padEnd = Uy.orderBy = Uy.omit = Uy.noop = Uy.min = Uy.merge = Uy.max = Uy.map = Uy.lowerFirst = Uy.lastIndexOf = Uy.last = Uy.keysIn = Uy.keys = Uy.join = Uy.isNaN = Uy.isMap = Uy.isInteger = Uy.isUndefined = Uy.isTypedArray = Uy.isSymbol = Uy.isString = Uy.isSafeInteger = Uy.isPlainObject = Uy.isObjectLike = void 0, 
Uy.toPairs = Uy.take = Uy.unionWith = Uy.flow = Uy.every = Uy.matchesProperty = Uy.isMatch = Uy.matches = Uy.updateWith = Uy.unset = Uy.parseInt = Uy.unzip = Uy.differenceWith = Uy.zip = Uy.result = Uy.meanBy = Uy.xorWith = Uy.curry = Uy.omitBy = Uy.differenceBy = Uy.template = Uy.templateSettings = Uy.unescape = Uy.escape = Uy.cloneDeepWith = Uy.capitalize = Uy.pickBy = Uy.replace = Uy.takeRight = Uy.size = Uy.memoize = Uy.identity = Uy.camelCase = Uy.lowerCase = Uy.kebabCase = Uy.snakeCase = Uy.uniqBy = Uy.constant = Uy.stubTrue = Uy.random = Uy.times = Uy.pullAll = Uy.nth = Uy.first = Uy.without = Uy.forOwn = Uy.get = Uy.keyBy = Uy.xor = Uy.values = void 0, 
Uy.isRegExp = Uy.toArray = Uy.split = Uy.minBy = Uy.mapValues = Uy.mapKeys = Uy.flatMap = Uy.intersectionWith = Uy.uniqWith = Uy.unionBy = Uy.sumBy = Uy.property = Uy.findKey = Uy.assignWith = Uy.some = Uy.upperCase = Uy.toFinite = Uy.toInteger = Uy.invoke = Uy.zipObject = Uy.mergeWith = Uy.truncate = Uy.fromPairs = void 0;

var pL = dL(zy);

Uy.assign = pL.default;

var hL = dL(N_);

Uy.assignIn = hL.default;

var vL = dL(B_);

Uy.castArray = vL.default;

var gL = dL(U_);

Uy.ceil = gL.default;

var mL = dL(CE);

Uy.chunk = mL.default;

var yL = dL(PE);

Uy.clamp = yL.default;

var _L = dL(TE);

Uy.clone = _L.default;

var EL = dL(Mw);

Uy.cloneDeep = EL.default;

var bL = dL(kw);

Uy.compact = bL.default;

var wL = dL(Rw);

Uy.concat = wL.default;

var DL = dL(Lw);

Uy.debounce = DL.default;

var SL = dL(Yw);

Uy.defaultTo = SL.default;

var AL = dL(Qw);

Uy.difference = AL.default;

var OL = dL(BS);

Uy.divide = OL.default;

var CL = dL(US);

Uy.drop = CL.default;

var xL = dL($S);

Uy.dropRight = xL.default;

var FL = dL(rA);

Uy.endsWith = FL.default;

var ML = dL(nA);

Uy.each = ML.default;

var PL = dL(ob);

Uy.eq = PL.default;

var IL = dL(cA);

Uy.filter = IL.default;

var kL = dL(mA);

Uy.find = kL.default;

var RL = dL(yA);

Uy.findIndex = RL.default;

var TL = dL(SA);

Uy.findLastIndex = TL.default;

var jL = dL(xA);

Uy.flatten = jL.default;

var LL = dL(RA);

Uy.flattenDeep = LL.default;

var NL = dL(yE);

Uy.floor = NL.default;

var BL = dL(oA);

Uy.forEach = BL.default;

var UL = dL(HA);

Uy.forIn = UL.default;

var zL = dL($A);

Uy.groupBy = zL.default;

var HL = dL(qA);

Uy.has = HL.default;

var $L = dL(dO);

Uy.head = $L.default;

var GL = dL(pO);

Uy.includes = GL.default;

var WL = dL(mO);

Uy.indexOf = WL.default;

var VL = dL(SO);

Uy.intersection = VL.default;

var KL = dL(xO);

Uy.invert = KL.default;

var qL = dL(q_);

Uy.isArray = qL.default;

var JL = dL(f_);

Uy.isArrayLike = JL.default;

var XL = dL(PS);

Uy.isArrayLikeObject = XL.default;

var ZL = dL(PO);

Uy.isBoolean = ZL.default;

var YL = dL(kO);

Uy.isBuffer = YL.default;

var QL = dL(jO);

Uy.isDate = QL.default;

var eN = dL(UO);

Uy.isEmpty = eN.default;

var tN = dL(qO);

Uy.isEqual = tN.default;

var rN = dL(gC);

Uy.isFinite = rN.default;

var nN = dL(mC);

Uy.isFunction = nN.default;

var oN = dL(S_);

Uy.isNil = oN.default;

var iN = dL(_C);

Uy.isNull = iN.default;

var aN = dL(EC);

Uy.isNumber = aN.default;

var sN = dL(qE);

Uy.isObject = sN.default;

var uN = dL(v_);

Uy.isObjectLike = uN.default;

var lN = dL(zO);

Uy.isPlainObject = lN.default;

var cN = dL(TA);

Uy.isString = cN.default;

var fN = dL(dE());

Uy.isSymbol = fN.default;

var dN = dL(h_);

Uy.isTypedArray = dN.default;

var pN = dL(wC);

Uy.isUndefined = pN.default;

var hN = dL(DC);

Uy.isInteger = hN.default;

var vN = dL(SC);

Uy.isMap = vN.default;

var gN = dL(z_);

Uy.isNaN = gN.default;

var mN = dL(AC);

Uy.join = mN.default;

var yN = dL(c_);

Uy.keys = yN.default;

var _N = dL(KE);

Uy.keysIn = _N.default;

var EN = dL(OC);

Uy.last = EN.default;

var bN = dL(AA);

Uy.lastIndexOf = bN.default;

var wN = dL(CC);

Uy.lowerFirst = wN.default;

var DN = dL(tD);

Uy.map = DN.default;

var SN = dL(xC);

Uy.max = SN.default;

var AN = dL(IC);

Uy.merge = AN.default;

var ON = dL(NC);

Uy.min = ON.default;

var CN = dL(VC);

Uy.noop = CN.default;

var xN = dL(KC);

Uy.omit = xN.default;

var FN = dL(Px);

Uy.orderBy = FN.default;

var MN = dL(Wx);

Uy.padEnd = MN.default;

var PN = dL(Jx);

Uy.padStart = PN.default;

var IN = dL(qC);

Uy.pick = IN.default;

var kN = dL(JC);

Uy.pull = kN.default;

var RN = dL(Qx);

Uy.pullAt = RN.default;

var TN = dL(rF);

Uy.range = TN.default;

var jN = dL(iF);

Uy.reduce = jN.default;

var LN = dL(cF);

Uy.remove = LN.default;

var NN = dL(dF);

Uy.reverse = NN.default;

var BN = dL(pF);

Uy.round = BN.default;

var UN = dL(yF);

Uy.set = UN.default;

var zN = dL(RF);

Uy.slice = zN.default;

var HN = dL(TF);

Uy.sortBy = HN.default;

var $N = dL(HF);

Uy.startsWith = $N.default;

var GN = dL($F);

Uy.sum = GN.default;

var WN = dL(GF);

Uy.throttle = WN.default;

var VN = dL(KF);

Uy.toLower = VN.default;

var KN = dL(H_);

Uy.toNumber = KN.default;

var qN = dL(YC);

Uy.toString = qN.default;

var JN = dL(ZF);

Uy.toUpper = JN.default;

var XN = dL(pE);

Uy.trim = XN.default;

var ZN = dL(tM);

Uy.trimEnd = ZN.default;

var YN = dL(nM);

Uy.trimStart = YN.default;

var QN = dL(iM);

Uy.uniq = QN.default;

var eB = dL(aM);

Uy.upperFirst = eB.default;

var tB = dL(cM);

Uy.union = tB.default;

var rB = dL(OM);

Uy.uniqueId = rB.default;

var nB = dL(hO);

Uy.values = nB.default;

var oB = dL(MM);

Uy.xor = oB.default;

var iB = dL(BM);

Uy.keyBy = iB.default;

var aB = dL(lE());

Uy.get = aB.default;

var sB = dL(UM);

Uy.forOwn = sB.default;

var uB = dL(GM);

Uy.without = uB.default;

var lB = dL(qM);

Uy.first = lB.default;

var cB = dL(ZM);

Uy.nth = cB.default;

var fB = dL(rP);

Uy.pullAll = fB.default;

var dB = dL(uP);

Uy.times = dB.default;

var pB = dL(pP);

Uy.random = pB.default;

var hB = dL(_P);

Uy.stubTrue = hB.default;

var vB = dL(EP);

Uy.constant = vB.default;

var gB = dL(bP);

Uy.uniqBy = gB.default;

var mB = dL(kP);

Uy.snakeCase = mB.default;

var yB = dL(pI);

Uy.kebabCase = yB.default;

var _B = dL(yI);

Uy.lowerCase = _B.default;

var EB = dL(DI);

Uy.camelCase = EB.default;

var bB = dL(W_);

Uy.identity = bB.default;

var wB = dL(XA);

Uy.memoize = wB.default;

var DB = dL(MI);

Uy.size = DB.default;

var SB = dL(RI);

Uy.takeRight = SB.default;

var AB = dL(TI);

Uy.replace = AB.default;

var OB = dL(jI);

Uy.pickBy = OB.default;

var CB = dL(HI);

Uy.capitalize = CB.default;

var xB = dL(VI);

Uy.cloneDeepWith = xB.default;

var FB = dL(JI);

Uy.escape = FB.default;

var MB = dL(ek);

Uy.unescape = MB.default;

var PB = dL(ik);

Uy.templateSettings = PB.default;

var IB = dL(fk);

Uy.template = IB.default;

var kB = dL(Fk);

Uy.differenceBy = kB.default;

var RB = dL(Tk);

Uy.omitBy = RB.default;

var TB = dL(jk);

Uy.curry = TB.default;

var jB = dL(PM);

Uy.xorWith = jB.default;

var LB = dL($k);

Uy.meanBy = LB.default;

var NB = dL(Gk);

Uy.result = NB.default;

var BB = dL(Kk);

Uy.zip = BB.default;

var UB = dL(Xk);

Uy.differenceWith = UB.default;

var zB = dL(tR);

Uy.unzip = zB.default;

var HB = dL(nR);

Uy.parseInt = HB.default;

var $B = dL(oR);

Uy.unset = $B.default;

var GB = dL(sR);

Uy.updateWith = GB.default;

var WB = dL(cR);

Uy.matches = WB.default;

var VB = dL(fR);

Uy.isMatch = VB.default;

var KB = dL(bR);

Uy.matchesProperty = KB.default;

var qB = dL(PR);

Uy.every = qB.default;

var JB = dL(UR);

Uy.flow = JB.default;

var XB = dL(WR);

Uy.unionWith = XB.default;

var ZB = dL(XR);

Uy.take = ZB.default;

var YB = dL(eT);

Uy.toPairs = YB.default;

var QB = dL(aT);

Uy.fromPairs = QB.default;

var eU = dL(sT);

Uy.truncate = eU.default;

var tU = dL(ET);

Uy.mergeWith = tU.default;

var rU = dL(DT);

Uy.zipObject = rU.default;

var nU = dL(xT);

Uy.invoke = nU.default;

var oU = dL(GS);

Uy.toInteger = oU.default;

var iU = dL(WS);

Uy.toFinite = iU.default;

var aU = dL(RT);

Uy.upperCase = aU.default;

var sU = dL(UT);

Uy.some = sU.default;

var uU = dL(qT);

Uy.assignWith = uU.default;

var lU = dL(ej);

Uy.findKey = lU.default;

var cU = dL(cE());

Uy.property = cU.default;

var fU = dL(sj);

Uy.sumBy = fU.default;

var dU = dL(pj);

Uy.unionBy = dU.default;

var pU = dL(Ej);

Uy.uniqWith = pU.default;

var hU = dL(Sj);

Uy.intersectionWith = hU.default;

var vU = dL(Ij);

Uy.flatMap = vU.default;

var gU = dL(Nj);

Uy.mapKeys = gU.default;

var mU = dL(Hj);

Uy.mapValues = mU.default;

var yU = dL(Vj);

Uy.minBy = yU.default;

var _U = dL(Xj);

Uy.split = _U.default;

var EU = dL(rL);

Uy.toArray = EU.default;

var bU = dL(uL);

Uy.isRegExp = bU.default;

var wU = dL(fL);

Uy.isSafeInteger = wU.default;

var DU = hL.default;

Uy.extend = DU, Uy.default = {
    assign: pL.default,
    assignIn: hL.default,
    extend: DU,
    castArray: vL.default,
    ceil: gL.default,
    chunk: mL.default,
    clamp: yL.default,
    clone: _L.default,
    cloneDeep: EL.default,
    compact: bL.default,
    concat: wL.default,
    debounce: DL.default,
    defaultTo: SL.default,
    difference: AL.default,
    divide: OL.default,
    drop: CL.default,
    dropRight: xL.default,
    endsWith: FL.default,
    each: ML.default,
    eq: PL.default,
    filter: IL.default,
    find: kL.default,
    findIndex: RL.default,
    findLastIndex: TL.default,
    flatten: jL.default,
    flattenDeep: LL.default,
    floor: NL.default,
    forEach: BL.default,
    forIn: UL.default,
    groupBy: zL.default,
    has: HL.default,
    head: $L.default,
    includes: GL.default,
    indexOf: WL.default,
    intersection: VL.default,
    invert: KL.default,
    isArray: qL.default,
    isArrayLike: JL.default,
    isArrayLikeObject: XL.default,
    isBoolean: ZL.default,
    isBuffer: YL.default,
    isDate: QL.default,
    isEmpty: eN.default,
    isEqual: tN.default,
    isFinite: rN.default,
    isFunction: nN.default,
    isNil: oN.default,
    isNull: iN.default,
    isNumber: aN.default,
    isObject: sN.default,
    isObjectLike: uN.default,
    isPlainObject: lN.default,
    isSafeInteger: wU.default,
    isString: cN.default,
    isSymbol: fN.default,
    isTypedArray: dN.default,
    isUndefined: pN.default,
    isInteger: hN.default,
    isMap: vN.default,
    isNaN: gN.default,
    join: mN.default,
    keys: yN.default,
    keysIn: _N.default,
    last: EN.default,
    lastIndexOf: bN.default,
    lowerFirst: wN.default,
    map: DN.default,
    max: SN.default,
    merge: AN.default,
    min: ON.default,
    noop: CN.default,
    omit: xN.default,
    orderBy: FN.default,
    padEnd: MN.default,
    padStart: PN.default,
    pick: IN.default,
    pull: kN.default,
    pullAt: RN.default,
    range: TN.default,
    reduce: jN.default,
    remove: LN.default,
    reverse: NN.default,
    round: BN.default,
    set: UN.default,
    slice: zN.default,
    sortBy: HN.default,
    startsWith: $N.default,
    sum: GN.default,
    throttle: WN.default,
    toLower: VN.default,
    toNumber: KN.default,
    toString: qN.default,
    toUpper: JN.default,
    trim: XN.default,
    trimEnd: ZN.default,
    trimStart: YN.default,
    uniq: QN.default,
    upperFirst: eB.default,
    union: tB.default,
    uniqueId: rB.default,
    values: nB.default,
    xor: oB.default,
    keyBy: iB.default,
    get: aB.default,
    forOwn: sB.default,
    without: uB.default,
    first: lB.default,
    nth: cB.default,
    pullAll: fB.default,
    times: dB.default,
    random: pB.default,
    stubTrue: hB.default,
    constant: vB.default,
    uniqBy: gB.default,
    snakeCase: mB.default,
    kebabCase: yB.default,
    lowerCase: _B.default,
    camelCase: EB.default,
    identity: bB.default,
    memoize: wB.default,
    size: DB.default,
    takeRight: SB.default,
    replace: AB.default,
    pickBy: OB.default,
    capitalize: CB.default,
    cloneDeepWith: xB.default,
    escape: FB.default,
    unescape: MB.default,
    templateSettings: PB.default,
    template: IB.default,
    differenceBy: kB.default,
    omitBy: RB.default,
    curry: TB.default,
    xorWith: jB.default,
    meanBy: LB.default,
    result: NB.default,
    zip: BB.default,
    differenceWith: UB.default,
    unzip: zB.default,
    parseInt: HB.default,
    unset: $B.default,
    updateWith: GB.default,
    matches: WB.default,
    isMatch: VB.default,
    matchesProperty: KB.default,
    every: qB.default,
    flow: JB.default,
    unionWith: XB.default,
    take: ZB.default,
    toPairs: YB.default,
    fromPairs: QB.default,
    truncate: eU.default,
    mergeWith: tU.default,
    zipObject: rU.default,
    invoke: nU.default,
    toInteger: oU.default,
    toFinite: iU.default,
    upperCase: aU.default,
    some: sU.default,
    assignWith: uU.default,
    findKey: lU.default,
    property: cU.default,
    sumBy: fU.default,
    unionBy: dU.default,
    uniqWith: pU.default,
    intersectionWith: hU.default,
    flatMap: vU.default,
    mapKeys: gU.default,
    mapValues: mU.default,
    minBy: yU.default,
    split: _U.default,
    toArray: EU.default,
    isRegExp: bU.default
};

var SU = {}, AU = {};

Object.defineProperty(AU, "__esModule", {
    value: !0
}), AU.getExtraConfig = AU.setExtraConfig = void 0;

let OU = new Map;

AU.setExtraConfig = function(e) {
    OU = e;
}, AU.getExtraConfig = function(e) {
    return OU.get(e);
};

var CU = {};

Object.defineProperty(CU, "__esModule", {
    value: !0
}), CU.ENABLE_SOURCE_MAPS = CU.ENABLE_OVERRIDES_DEPENDENCY_MAP = CU.INCREMENTAL_OPTIMIZATION = CU.INCREMENTAL_INPUT_OUTPUT_CACHE = CU.LOG_LEVEL = CU.ANALYZE = CU.PARALLEL = CU.INCREMENTAL = CU.DAEMON = CU.DOT = CU.PROPERTIES = CU.HVIGOR_MEMORY_THRESHOLD = CU.OHOS_ARK_COMPILE_SOURCE_MAP_DIR = CU.HVIGOR_ENABLE_MEMORY_CACHE = CU.OHOS_ARK_COMPILE_MAX_SIZE = CU.HVIGOR_POOL_CACHE_TTL = CU.HVIGOR_POOL_CACHE_CAPACITY = CU.HVIGOR_POOL_MAX_CORE_SIZE = CU.HVIGOR_POOL_MAX_SIZE = CU.BUILD_CACHE_DIR = CU.ENABLE_SIGN_TASK_KEY = CU.HVIGOR_CACHE_DIR_KEY = CU.WORK_SPACE = CU.PROJECT_CACHES = CU.HVIGOR_USER_HOME_DIR_NAME = CU.DEFAULT_PACKAGE_JSON = CU.DEFAULT_OH_PACKAGE_JSON_FILE_NAME = CU.DEFAULT_HVIGOR_CONFIG_JSON_FILE_NAME = CU.PNPM = CU.HVIGOR = CU.NPM_TOOL = CU.PNPM_TOOL = CU.HVIGOR_ENGINE_PACKAGE_NAME = void 0;

const xU = y;

CU.HVIGOR_ENGINE_PACKAGE_NAME = "@ohos/hvigor", CU.PNPM_TOOL = (0, xU.isWindows)() ? "pnpm.cmd" : "pnpm", 
CU.NPM_TOOL = (0, xU.isWindows)() ? "npm.cmd" : "npm", CU.HVIGOR = "hvigor", CU.PNPM = "pnpm", 
CU.DEFAULT_HVIGOR_CONFIG_JSON_FILE_NAME = "hvigor-config.json5", CU.DEFAULT_OH_PACKAGE_JSON_FILE_NAME = "oh-package.json5", 
CU.DEFAULT_PACKAGE_JSON = "package.json", CU.HVIGOR_USER_HOME_DIR_NAME = ".hvigor", 
CU.PROJECT_CACHES = "project_caches", CU.WORK_SPACE = "workspace", CU.HVIGOR_CACHE_DIR_KEY = "hvigor.cacheDir", 
CU.ENABLE_SIGN_TASK_KEY = "enableSignTask", CU.BUILD_CACHE_DIR = "build-cache-dir", 
CU.HVIGOR_POOL_MAX_SIZE = "hvigor.pool.maxSize", CU.HVIGOR_POOL_MAX_CORE_SIZE = "hvigor.pool.maxCoreSize", 
CU.HVIGOR_POOL_CACHE_CAPACITY = "hvigor.pool.cache.capacity", CU.HVIGOR_POOL_CACHE_TTL = "hvigor.pool.cache.ttl", 
CU.OHOS_ARK_COMPILE_MAX_SIZE = "ohos.arkCompile.maxSize", CU.HVIGOR_ENABLE_MEMORY_CACHE = "hvigor.enableMemoryCache", 
CU.OHOS_ARK_COMPILE_SOURCE_MAP_DIR = "ohos.arkCompile.sourceMapDir", CU.HVIGOR_MEMORY_THRESHOLD = "hvigor.memoryThreshold", 
CU.PROPERTIES = "properties", CU.DOT = ".", CU.DAEMON = "daemon", CU.INCREMENTAL = "incremental", 
CU.PARALLEL = "typeCheck", CU.ANALYZE = "analyze", CU.LOG_LEVEL = "logLevel", CU.INCREMENTAL_INPUT_OUTPUT_CACHE = "hvigor.incremental.optimization", 
CU.INCREMENTAL_OPTIMIZATION = "hvigor.task.schedule.optimization", CU.ENABLE_OVERRIDES_DEPENDENCY_MAP = "enableOverridesDependencyMap", 
CU.ENABLE_SOURCE_MAPS = "--enable-source-maps";

var FU = {}, MU = {}, PU = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(MU, "__esModule", {
    value: !0
}), MU.getHvigorUserHomeCacheDir = void 0;

const IU = PU(Bs), kU = PU(a), RU = PU(r), TU = CU;

MU.getHvigorUserHomeCacheDir = function() {
    const e = RU.default.resolve(kU.default.homedir(), TU.HVIGOR_USER_HOME_DIR_NAME), t = process.env.HVIGOR_USER_HOME;
    return void 0 !== t && RU.default.isAbsolute(t) ? (IU.default.ensureDirSync(t), 
    t) : e;
}, function(e) {
    var t = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.HVIGOR_CONFIG_SCHEMA_PATH = e.HVIGOR_PROJECT_WRAPPER_HOME = e.HVIGOR_PROJECT_ROOT_DIR = e.HVIGOR_PROJECT_CACHES_HOME = e.HVIGOR_PNPM_STORE_PATH = e.HVIGOR_WRAPPER_PNPM_SCRIPT_PATH = e.HVIGOR_WRAPPER_TOOLS_HOME = e.HVIGOR_USER_HOME = void 0;
    const n = t(r), o = MU, i = CU;
    e.HVIGOR_USER_HOME = (0, o.getHvigorUserHomeCacheDir)(), e.HVIGOR_WRAPPER_TOOLS_HOME = n.default.resolve(e.HVIGOR_USER_HOME, "wrapper", "tools"), 
    e.HVIGOR_WRAPPER_PNPM_SCRIPT_PATH = n.default.resolve(e.HVIGOR_WRAPPER_TOOLS_HOME, "node_modules", ".bin", i.PNPM_TOOL), 
    e.HVIGOR_PNPM_STORE_PATH = n.default.resolve(e.HVIGOR_USER_HOME, "caches"), e.HVIGOR_PROJECT_CACHES_HOME = n.default.resolve(e.HVIGOR_USER_HOME, i.PROJECT_CACHES), 
    e.HVIGOR_PROJECT_ROOT_DIR = process.cwd(), e.HVIGOR_PROJECT_WRAPPER_HOME = n.default.resolve(e.HVIGOR_PROJECT_ROOT_DIR, i.HVIGOR), 
    e.HVIGOR_CONFIG_SCHEMA_PATH = n.default.resolve(__dirname, "../../../res/hvigor-config-schema.json");
}(FU);

var jU = {}, LU = {};

!function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.coreParameter = e.defaultProperties = e.defaultStartParam = e.LogLevelMap = e.AnalyzeModeKeyMap = e.OldAnalyzeModeMap = e.AnalyzeModeMap = e.OptimizationStrategy = e.AnalyzeMode = e.CoreParameter = void 0;
    const t = Gm;
    class r {
        constructor() {
            this._properties = {}, this._extParams = {}, this._startParams = {
                ...e.defaultStartParam
            }, this._workspaceDir = "";
        }
        get properties() {
            return this._properties;
        }
        set properties(e) {
            this._properties = e;
        }
        get extParams() {
            return this._extParams;
        }
        set extParams(e) {
            this._extParams = e;
        }
        get startParams() {
            return this._startParams;
        }
        get workspaceDir() {
            return this._workspaceDir;
        }
        set workspaceDir(e) {
            this._workspaceDir = e;
        }
        clean() {
            this._properties = {}, this._extParams = {}, this._startParams = {
                ...e.defaultStartParam
            }, this._workspaceDir = "";
        }
    }
    var n, o;
    e.CoreParameter = r, function(e) {
        e[e.NORMAL = 0] = "NORMAL", e[e.ADVANCED = 1] = "ADVANCED", e[e.FALSE = 2] = "FALSE", 
        e[e.TRACE = 3] = "TRACE";
    }(n = e.AnalyzeMode || (e.AnalyzeMode = {})), function(e) {
        e.PERFORMANCE = "performance", e.MEMORY = "memory";
    }(o = e.OptimizationStrategy || (e.OptimizationStrategy = {})), e.AnalyzeModeMap = new Map([ [ "default", n.NORMAL ], [ "verbose", n.ADVANCED ], [ !1, n.FALSE ], [ "false", n.FALSE ], [ "normal", n.NORMAL ], [ "advanced", n.ADVANCED ], [ "ultrafine", n.TRACE ] ]), 
    e.OldAnalyzeModeMap = new Map([ [ "default", "normal" ], [ "verbose", "advanced" ] ]), 
    e.AnalyzeModeKeyMap = new Map([ [ n.NORMAL, "normal" ], [ n.ADVANCED, "advanced" ], [ n.TRACE, "ultrafine" ], [ n.FALSE, !1 ] ]), 
    e.LogLevelMap = new Map([ [ "info", t.levels.INFO ], [ 'debug"', t.levels.DEBUG ], [ 'warn"', t.levels.WARN ], [ 'error"', t.levels.ERROR ] ]), 
    e.defaultStartParam = {
        hvigorfileTypeCheck: !1,
        parallelExecution: !0,
        incrementalExecution: !0,
        printStackTrace: !1,
        daemon: !0,
        analyze: n.NORMAL,
        logLevel: t.levels.INFO,
        optimizationStrategy: o.MEMORY,
        hotCompile: !1,
        hotReloadBuild: !1
    }, e.defaultProperties = {
        enableSignTask: !0,
        skipNativeIncremental: !1,
        "hvigor.keepDependency": !0
    }, e.coreParameter = new r;
}(LU);

var NU = {}, BU = {}, UU = {};

Object.defineProperty(UU, "__esModule", {
    value: !0
}), UU.Unicode = void 0;

class zU {}

UU.Unicode = zU, zU.SPACE_SEPARATOR = /[\u1680\u2000-\u200A\u202F\u205F\u3000]/, 
zU.ID_START = /[\xAA\xB5\xBA\xC0-\xD6\xD8-\xF6\xF8-\u02C1\u02C6-\u02D1\u02E0-\u02E4\u02EC\u02EE\u0370-\u0374\u0376\u0377\u037A-\u037D\u037F\u0386\u0388-\u038A\u038C\u038E-\u03A1\u03A3-\u03F5\u03F7-\u0481\u048A-\u052F\u0531-\u0556\u0559\u0561-\u0587\u05D0-\u05EA\u05F0-\u05F2\u0620-\u064A\u066E\u066F\u0671-\u06D3\u06D5\u06E5\u06E6\u06EE\u06EF\u06FA-\u06FC\u06FF\u0710\u0712-\u072F\u074D-\u07A5\u07B1\u07CA-\u07EA\u07F4\u07F5\u07FA\u0800-\u0815\u081A\u0824\u0828\u0840-\u0858\u0860-\u086A\u08A0-\u08B4\u08B6-\u08BD\u0904-\u0939\u093D\u0950\u0958-\u0961\u0971-\u0980\u0985-\u098C\u098F\u0990\u0993-\u09A8\u09AA-\u09B0\u09B2\u09B6-\u09B9\u09BD\u09CE\u09DC\u09DD\u09DF-\u09E1\u09F0\u09F1\u09FC\u0A05-\u0A0A\u0A0F\u0A10\u0A13-\u0A28\u0A2A-\u0A30\u0A32\u0A33\u0A35\u0A36\u0A38\u0A39\u0A59-\u0A5C\u0A5E\u0A72-\u0A74\u0A85-\u0A8D\u0A8F-\u0A91\u0A93-\u0AA8\u0AAA-\u0AB0\u0AB2\u0AB3\u0AB5-\u0AB9\u0ABD\u0AD0\u0AE0\u0AE1\u0AF9\u0B05-\u0B0C\u0B0F\u0B10\u0B13-\u0B28\u0B2A-\u0B30\u0B32\u0B33\u0B35-\u0B39\u0B3D\u0B5C\u0B5D\u0B5F-\u0B61\u0B71\u0B83\u0B85-\u0B8A\u0B8E-\u0B90\u0B92-\u0B95\u0B99\u0B9A\u0B9C\u0B9E\u0B9F\u0BA3\u0BA4\u0BA8-\u0BAA\u0BAE-\u0BB9\u0BD0\u0C05-\u0C0C\u0C0E-\u0C10\u0C12-\u0C28\u0C2A-\u0C39\u0C3D\u0C58-\u0C5A\u0C60\u0C61\u0C80\u0C85-\u0C8C\u0C8E-\u0C90\u0C92-\u0CA8\u0CAA-\u0CB3\u0CB5-\u0CB9\u0CBD\u0CDE\u0CE0\u0CE1\u0CF1\u0CF2\u0D05-\u0D0C\u0D0E-\u0D10\u0D12-\u0D3A\u0D3D\u0D4E\u0D54-\u0D56\u0D5F-\u0D61\u0D7A-\u0D7F\u0D85-\u0D96\u0D9A-\u0DB1\u0DB3-\u0DBB\u0DBD\u0DC0-\u0DC6\u0E01-\u0E30\u0E32\u0E33\u0E40-\u0E46\u0E81\u0E82\u0E84\u0E87\u0E88\u0E8A\u0E8D\u0E94-\u0E97\u0E99-\u0E9F\u0EA1-\u0EA3\u0EA5\u0EA7\u0EAA\u0EAB\u0EAD-\u0EB0\u0EB2\u0EB3\u0EBD\u0EC0-\u0EC4\u0EC6\u0EDC-\u0EDF\u0F00\u0F40-\u0F47\u0F49-\u0F6C\u0F88-\u0F8C\u1000-\u102A\u103F\u1050-\u1055\u105A-\u105D\u1061\u1065\u1066\u106E-\u1070\u1075-\u1081\u108E\u10A0-\u10C5\u10C7\u10CD\u10D0-\u10FA\u10FC-\u1248\u124A-\u124D\u1250-\u1256\u1258\u125A-\u125D\u1260-\u1288\u128A-\u128D\u1290-\u12B0\u12B2-\u12B5\u12B8-\u12BE\u12C0\u12C2-\u12C5\u12C8-\u12D6\u12D8-\u1310\u1312-\u1315\u1318-\u135A\u1380-\u138F\u13A0-\u13F5\u13F8-\u13FD\u1401-\u166C\u166F-\u167F\u1681-\u169A\u16A0-\u16EA\u16EE-\u16F8\u1700-\u170C\u170E-\u1711\u1720-\u1731\u1740-\u1751\u1760-\u176C\u176E-\u1770\u1780-\u17B3\u17D7\u17DC\u1820-\u1877\u1880-\u1884\u1887-\u18A8\u18AA\u18B0-\u18F5\u1900-\u191E\u1950-\u196D\u1970-\u1974\u1980-\u19AB\u19B0-\u19C9\u1A00-\u1A16\u1A20-\u1A54\u1AA7\u1B05-\u1B33\u1B45-\u1B4B\u1B83-\u1BA0\u1BAE\u1BAF\u1BBA-\u1BE5\u1C00-\u1C23\u1C4D-\u1C4F\u1C5A-\u1C7D\u1C80-\u1C88\u1CE9-\u1CEC\u1CEE-\u1CF1\u1CF5\u1CF6\u1D00-\u1DBF\u1E00-\u1F15\u1F18-\u1F1D\u1F20-\u1F45\u1F48-\u1F4D\u1F50-\u1F57\u1F59\u1F5B\u1F5D\u1F5F-\u1F7D\u1F80-\u1FB4\u1FB6-\u1FBC\u1FBE\u1FC2-\u1FC4\u1FC6-\u1FCC\u1FD0-\u1FD3\u1FD6-\u1FDB\u1FE0-\u1FEC\u1FF2-\u1FF4\u1FF6-\u1FFC\u2071\u207F\u2090-\u209C\u2102\u2107\u210A-\u2113\u2115\u2119-\u211D\u2124\u2126\u2128\u212A-\u212D\u212F-\u2139\u213C-\u213F\u2145-\u2149\u214E\u2160-\u2188\u2C00-\u2C2E\u2C30-\u2C5E\u2C60-\u2CE4\u2CEB-\u2CEE\u2CF2\u2CF3\u2D00-\u2D25\u2D27\u2D2D\u2D30-\u2D67\u2D6F\u2D80-\u2D96\u2DA0-\u2DA6\u2DA8-\u2DAE\u2DB0-\u2DB6\u2DB8-\u2DBE\u2DC0-\u2DC6\u2DC8-\u2DCE\u2DD0-\u2DD6\u2DD8-\u2DDE\u2E2F\u3005-\u3007\u3021-\u3029\u3031-\u3035\u3038-\u303C\u3041-\u3096\u309D-\u309F\u30A1-\u30FA\u30FC-\u30FF\u3105-\u312E\u3131-\u318E\u31A0-\u31BA\u31F0-\u31FF\u3400-\u4DB5\u4E00-\u9FEA\uA000-\uA48C\uA4D0-\uA4FD\uA500-\uA60C\uA610-\uA61F\uA62A\uA62B\uA640-\uA66E\uA67F-\uA69D\uA6A0-\uA6EF\uA717-\uA71F\uA722-\uA788\uA78B-\uA7AE\uA7B0-\uA7B7\uA7F7-\uA801\uA803-\uA805\uA807-\uA80A\uA80C-\uA822\uA840-\uA873\uA882-\uA8B3\uA8F2-\uA8F7\uA8FB\uA8FD\uA90A-\uA925\uA930-\uA946\uA960-\uA97C\uA984-\uA9B2\uA9CF\uA9E0-\uA9E4\uA9E6-\uA9EF\uA9FA-\uA9FE\uAA00-\uAA28\uAA40-\uAA42\uAA44-\uAA4B\uAA60-\uAA76\uAA7A\uAA7E-\uAAAF\uAAB1\uAAB5\uAAB6\uAAB9-\uAABD\uAAC0\uAAC2\uAADB-\uAADD\uAAE0-\uAAEA\uAAF2-\uAAF4\uAB01-\uAB06\uAB09-\uAB0E\uAB11-\uAB16\uAB20-\uAB26\uAB28-\uAB2E\uAB30-\uAB5A\uAB5C-\uAB65\uAB70-\uABE2\uAC00-\uD7A3\uD7B0-\uD7C6\uD7CB-\uD7FB\uF900-\uFA6D\uFA70-\uFAD9\uFB00-\uFB06\uFB13-\uFB17\uFB1D\uFB1F-\uFB28\uFB2A-\uFB36\uFB38-\uFB3C\uFB3E\uFB40\uFB41\uFB43\uFB44\uFB46-\uFBB1\uFBD3-\uFD3D\uFD50-\uFD8F\uFD92-\uFDC7\uFDF0-\uFDFB\uFE70-\uFE74\uFE76-\uFEFC\uFF21-\uFF3A\uFF41-\uFF5A\uFF66-\uFFBE\uFFC2-\uFFC7\uFFCA-\uFFCF\uFFD2-\uFFD7\uFFDA-\uFFDC]|\uD800[\uDC00-\uDC0B\uDC0D-\uDC26\uDC28-\uDC3A\uDC3C\uDC3D\uDC3F-\uDC4D\uDC50-\uDC5D\uDC80-\uDCFA\uDD40-\uDD74\uDE80-\uDE9C\uDEA0-\uDED0\uDF00-\uDF1F\uDF2D-\uDF4A\uDF50-\uDF75\uDF80-\uDF9D\uDFA0-\uDFC3\uDFC8-\uDFCF\uDFD1-\uDFD5]|\uD801[\uDC00-\uDC9D\uDCB0-\uDCD3\uDCD8-\uDCFB\uDD00-\uDD27\uDD30-\uDD63\uDE00-\uDF36\uDF40-\uDF55\uDF60-\uDF67]|\uD802[\uDC00-\uDC05\uDC08\uDC0A-\uDC35\uDC37\uDC38\uDC3C\uDC3F-\uDC55\uDC60-\uDC76\uDC80-\uDC9E\uDCE0-\uDCF2\uDCF4\uDCF5\uDD00-\uDD15\uDD20-\uDD39\uDD80-\uDDB7\uDDBE\uDDBF\uDE00\uDE10-\uDE13\uDE15-\uDE17\uDE19-\uDE33\uDE60-\uDE7C\uDE80-\uDE9C\uDEC0-\uDEC7\uDEC9-\uDEE4\uDF00-\uDF35\uDF40-\uDF55\uDF60-\uDF72\uDF80-\uDF91]|\uD803[\uDC00-\uDC48\uDC80-\uDCB2\uDCC0-\uDCF2]|\uD804[\uDC03-\uDC37\uDC83-\uDCAF\uDCD0-\uDCE8\uDD03-\uDD26\uDD50-\uDD72\uDD76\uDD83-\uDDB2\uDDC1-\uDDC4\uDDDA\uDDDC\uDE00-\uDE11\uDE13-\uDE2B\uDE80-\uDE86\uDE88\uDE8A-\uDE8D\uDE8F-\uDE9D\uDE9F-\uDEA8\uDEB0-\uDEDE\uDF05-\uDF0C\uDF0F\uDF10\uDF13-\uDF28\uDF2A-\uDF30\uDF32\uDF33\uDF35-\uDF39\uDF3D\uDF50\uDF5D-\uDF61]|\uD805[\uDC00-\uDC34\uDC47-\uDC4A\uDC80-\uDCAF\uDCC4\uDCC5\uDCC7\uDD80-\uDDAE\uDDD8-\uDDDB\uDE00-\uDE2F\uDE44\uDE80-\uDEAA\uDF00-\uDF19]|\uD806[\uDCA0-\uDCDF\uDCFF\uDE00\uDE0B-\uDE32\uDE3A\uDE50\uDE5C-\uDE83\uDE86-\uDE89\uDEC0-\uDEF8]|\uD807[\uDC00-\uDC08\uDC0A-\uDC2E\uDC40\uDC72-\uDC8F\uDD00-\uDD06\uDD08\uDD09\uDD0B-\uDD30\uDD46]|\uD808[\uDC00-\uDF99]|\uD809[\uDC00-\uDC6E\uDC80-\uDD43]|[\uD80C\uD81C-\uD820\uD840-\uD868\uD86A-\uD86C\uD86F-\uD872\uD874-\uD879][\uDC00-\uDFFF]|\uD80D[\uDC00-\uDC2E]|\uD811[\uDC00-\uDE46]|\uD81A[\uDC00-\uDE38\uDE40-\uDE5E\uDED0-\uDEED\uDF00-\uDF2F\uDF40-\uDF43\uDF63-\uDF77\uDF7D-\uDF8F]|\uD81B[\uDF00-\uDF44\uDF50\uDF93-\uDF9F\uDFE0\uDFE1]|\uD821[\uDC00-\uDFEC]|\uD822[\uDC00-\uDEF2]|\uD82C[\uDC00-\uDD1E\uDD70-\uDEFB]|\uD82F[\uDC00-\uDC6A\uDC70-\uDC7C\uDC80-\uDC88\uDC90-\uDC99]|\uD835[\uDC00-\uDC54\uDC56-\uDC9C\uDC9E\uDC9F\uDCA2\uDCA5\uDCA6\uDCA9-\uDCAC\uDCAE-\uDCB9\uDCBB\uDCBD-\uDCC3\uDCC5-\uDD05\uDD07-\uDD0A\uDD0D-\uDD14\uDD16-\uDD1C\uDD1E-\uDD39\uDD3B-\uDD3E\uDD40-\uDD44\uDD46\uDD4A-\uDD50\uDD52-\uDEA5\uDEA8-\uDEC0\uDEC2-\uDEDA\uDEDC-\uDEFA\uDEFC-\uDF14\uDF16-\uDF34\uDF36-\uDF4E\uDF50-\uDF6E\uDF70-\uDF88\uDF8A-\uDFA8\uDFAA-\uDFC2\uDFC4-\uDFCB]|\uD83A[\uDC00-\uDCC4\uDD00-\uDD43]|\uD83B[\uDE00-\uDE03\uDE05-\uDE1F\uDE21\uDE22\uDE24\uDE27\uDE29-\uDE32\uDE34-\uDE37\uDE39\uDE3B\uDE42\uDE47\uDE49\uDE4B\uDE4D-\uDE4F\uDE51\uDE52\uDE54\uDE57\uDE59\uDE5B\uDE5D\uDE5F\uDE61\uDE62\uDE64\uDE67-\uDE6A\uDE6C-\uDE72\uDE74-\uDE77\uDE79-\uDE7C\uDE7E\uDE80-\uDE89\uDE8B-\uDE9B\uDEA1-\uDEA3\uDEA5-\uDEA9\uDEAB-\uDEBB]|\uD869[\uDC00-\uDED6\uDF00-\uDFFF]|\uD86D[\uDC00-\uDF34\uDF40-\uDFFF]|\uD86E[\uDC00-\uDC1D\uDC20-\uDFFF]|\uD873[\uDC00-\uDEA1\uDEB0-\uDFFF]|\uD87A[\uDC00-\uDFE0]|\uD87E[\uDC00-\uDE1D]/, 
zU.ID_CONTINUE = /[\xAA\xB5\xBA\xC0-\xD6\xD8-\xF6\xF8-\u02C1\u02C6-\u02D1\u02E0-\u02E4\u02EC\u02EE\u0300-\u0374\u0376\u0377\u037A-\u037D\u037F\u0386\u0388-\u038A\u038C\u038E-\u03A1\u03A3-\u03F5\u03F7-\u0481\u0483-\u0487\u048A-\u052F\u0531-\u0556\u0559\u0561-\u0587\u0591-\u05BD\u05BF\u05C1\u05C2\u05C4\u05C5\u05C7\u05D0-\u05EA\u05F0-\u05F2\u0610-\u061A\u0620-\u0669\u066E-\u06D3\u06D5-\u06DC\u06DF-\u06E8\u06EA-\u06FC\u06FF\u0710-\u074A\u074D-\u07B1\u07C0-\u07F5\u07FA\u0800-\u082D\u0840-\u085B\u0860-\u086A\u08A0-\u08B4\u08B6-\u08BD\u08D4-\u08E1\u08E3-\u0963\u0966-\u096F\u0971-\u0983\u0985-\u098C\u098F\u0990\u0993-\u09A8\u09AA-\u09B0\u09B2\u09B6-\u09B9\u09BC-\u09C4\u09C7\u09C8\u09CB-\u09CE\u09D7\u09DC\u09DD\u09DF-\u09E3\u09E6-\u09F1\u09FC\u0A01-\u0A03\u0A05-\u0A0A\u0A0F\u0A10\u0A13-\u0A28\u0A2A-\u0A30\u0A32\u0A33\u0A35\u0A36\u0A38\u0A39\u0A3C\u0A3E-\u0A42\u0A47\u0A48\u0A4B-\u0A4D\u0A51\u0A59-\u0A5C\u0A5E\u0A66-\u0A75\u0A81-\u0A83\u0A85-\u0A8D\u0A8F-\u0A91\u0A93-\u0AA8\u0AAA-\u0AB0\u0AB2\u0AB3\u0AB5-\u0AB9\u0ABC-\u0AC5\u0AC7-\u0AC9\u0ACB-\u0ACD\u0AD0\u0AE0-\u0AE3\u0AE6-\u0AEF\u0AF9-\u0AFF\u0B01-\u0B03\u0B05-\u0B0C\u0B0F\u0B10\u0B13-\u0B28\u0B2A-\u0B30\u0B32\u0B33\u0B35-\u0B39\u0B3C-\u0B44\u0B47\u0B48\u0B4B-\u0B4D\u0B56\u0B57\u0B5C\u0B5D\u0B5F-\u0B63\u0B66-\u0B6F\u0B71\u0B82\u0B83\u0B85-\u0B8A\u0B8E-\u0B90\u0B92-\u0B95\u0B99\u0B9A\u0B9C\u0B9E\u0B9F\u0BA3\u0BA4\u0BA8-\u0BAA\u0BAE-\u0BB9\u0BBE-\u0BC2\u0BC6-\u0BC8\u0BCA-\u0BCD\u0BD0\u0BD7\u0BE6-\u0BEF\u0C00-\u0C03\u0C05-\u0C0C\u0C0E-\u0C10\u0C12-\u0C28\u0C2A-\u0C39\u0C3D-\u0C44\u0C46-\u0C48\u0C4A-\u0C4D\u0C55\u0C56\u0C58-\u0C5A\u0C60-\u0C63\u0C66-\u0C6F\u0C80-\u0C83\u0C85-\u0C8C\u0C8E-\u0C90\u0C92-\u0CA8\u0CAA-\u0CB3\u0CB5-\u0CB9\u0CBC-\u0CC4\u0CC6-\u0CC8\u0CCA-\u0CCD\u0CD5\u0CD6\u0CDE\u0CE0-\u0CE3\u0CE6-\u0CEF\u0CF1\u0CF2\u0D00-\u0D03\u0D05-\u0D0C\u0D0E-\u0D10\u0D12-\u0D44\u0D46-\u0D48\u0D4A-\u0D4E\u0D54-\u0D57\u0D5F-\u0D63\u0D66-\u0D6F\u0D7A-\u0D7F\u0D82\u0D83\u0D85-\u0D96\u0D9A-\u0DB1\u0DB3-\u0DBB\u0DBD\u0DC0-\u0DC6\u0DCA\u0DCF-\u0DD4\u0DD6\u0DD8-\u0DDF\u0DE6-\u0DEF\u0DF2\u0DF3\u0E01-\u0E3A\u0E40-\u0E4E\u0E50-\u0E59\u0E81\u0E82\u0E84\u0E87\u0E88\u0E8A\u0E8D\u0E94-\u0E97\u0E99-\u0E9F\u0EA1-\u0EA3\u0EA5\u0EA7\u0EAA\u0EAB\u0EAD-\u0EB9\u0EBB-\u0EBD\u0EC0-\u0EC4\u0EC6\u0EC8-\u0ECD\u0ED0-\u0ED9\u0EDC-\u0EDF\u0F00\u0F18\u0F19\u0F20-\u0F29\u0F35\u0F37\u0F39\u0F3E-\u0F47\u0F49-\u0F6C\u0F71-\u0F84\u0F86-\u0F97\u0F99-\u0FBC\u0FC6\u1000-\u1049\u1050-\u109D\u10A0-\u10C5\u10C7\u10CD\u10D0-\u10FA\u10FC-\u1248\u124A-\u124D\u1250-\u1256\u1258\u125A-\u125D\u1260-\u1288\u128A-\u128D\u1290-\u12B0\u12B2-\u12B5\u12B8-\u12BE\u12C0\u12C2-\u12C5\u12C8-\u12D6\u12D8-\u1310\u1312-\u1315\u1318-\u135A\u135D-\u135F\u1380-\u138F\u13A0-\u13F5\u13F8-\u13FD\u1401-\u166C\u166F-\u167F\u1681-\u169A\u16A0-\u16EA\u16EE-\u16F8\u1700-\u170C\u170E-\u1714\u1720-\u1734\u1740-\u1753\u1760-\u176C\u176E-\u1770\u1772\u1773\u1780-\u17D3\u17D7\u17DC\u17DD\u17E0-\u17E9\u180B-\u180D\u1810-\u1819\u1820-\u1877\u1880-\u18AA\u18B0-\u18F5\u1900-\u191E\u1920-\u192B\u1930-\u193B\u1946-\u196D\u1970-\u1974\u1980-\u19AB\u19B0-\u19C9\u19D0-\u19D9\u1A00-\u1A1B\u1A20-\u1A5E\u1A60-\u1A7C\u1A7F-\u1A89\u1A90-\u1A99\u1AA7\u1AB0-\u1ABD\u1B00-\u1B4B\u1B50-\u1B59\u1B6B-\u1B73\u1B80-\u1BF3\u1C00-\u1C37\u1C40-\u1C49\u1C4D-\u1C7D\u1C80-\u1C88\u1CD0-\u1CD2\u1CD4-\u1CF9\u1D00-\u1DF9\u1DFB-\u1F15\u1F18-\u1F1D\u1F20-\u1F45\u1F48-\u1F4D\u1F50-\u1F57\u1F59\u1F5B\u1F5D\u1F5F-\u1F7D\u1F80-\u1FB4\u1FB6-\u1FBC\u1FBE\u1FC2-\u1FC4\u1FC6-\u1FCC\u1FD0-\u1FD3\u1FD6-\u1FDB\u1FE0-\u1FEC\u1FF2-\u1FF4\u1FF6-\u1FFC\u203F\u2040\u2054\u2071\u207F\u2090-\u209C\u20D0-\u20DC\u20E1\u20E5-\u20F0\u2102\u2107\u210A-\u2113\u2115\u2119-\u211D\u2124\u2126\u2128\u212A-\u212D\u212F-\u2139\u213C-\u213F\u2145-\u2149\u214E\u2160-\u2188\u2C00-\u2C2E\u2C30-\u2C5E\u2C60-\u2CE4\u2CEB-\u2CF3\u2D00-\u2D25\u2D27\u2D2D\u2D30-\u2D67\u2D6F\u2D7F-\u2D96\u2DA0-\u2DA6\u2DA8-\u2DAE\u2DB0-\u2DB6\u2DB8-\u2DBE\u2DC0-\u2DC6\u2DC8-\u2DCE\u2DD0-\u2DD6\u2DD8-\u2DDE\u2DE0-\u2DFF\u2E2F\u3005-\u3007\u3021-\u302F\u3031-\u3035\u3038-\u303C\u3041-\u3096\u3099\u309A\u309D-\u309F\u30A1-\u30FA\u30FC-\u30FF\u3105-\u312E\u3131-\u318E\u31A0-\u31BA\u31F0-\u31FF\u3400-\u4DB5\u4E00-\u9FEA\uA000-\uA48C\uA4D0-\uA4FD\uA500-\uA60C\uA610-\uA62B\uA640-\uA66F\uA674-\uA67D\uA67F-\uA6F1\uA717-\uA71F\uA722-\uA788\uA78B-\uA7AE\uA7B0-\uA7B7\uA7F7-\uA827\uA840-\uA873\uA880-\uA8C5\uA8D0-\uA8D9\uA8E0-\uA8F7\uA8FB\uA8FD\uA900-\uA92D\uA930-\uA953\uA960-\uA97C\uA980-\uA9C0\uA9CF-\uA9D9\uA9E0-\uA9FE\uAA00-\uAA36\uAA40-\uAA4D\uAA50-\uAA59\uAA60-\uAA76\uAA7A-\uAAC2\uAADB-\uAADD\uAAE0-\uAAEF\uAAF2-\uAAF6\uAB01-\uAB06\uAB09-\uAB0E\uAB11-\uAB16\uAB20-\uAB26\uAB28-\uAB2E\uAB30-\uAB5A\uAB5C-\uAB65\uAB70-\uABEA\uABEC\uABED\uABF0-\uABF9\uAC00-\uD7A3\uD7B0-\uD7C6\uD7CB-\uD7FB\uF900-\uFA6D\uFA70-\uFAD9\uFB00-\uFB06\uFB13-\uFB17\uFB1D-\uFB28\uFB2A-\uFB36\uFB38-\uFB3C\uFB3E\uFB40\uFB41\uFB43\uFB44\uFB46-\uFBB1\uFBD3-\uFD3D\uFD50-\uFD8F\uFD92-\uFDC7\uFDF0-\uFDFB\uFE00-\uFE0F\uFE20-\uFE2F\uFE33\uFE34\uFE4D-\uFE4F\uFE70-\uFE74\uFE76-\uFEFC\uFF10-\uFF19\uFF21-\uFF3A\uFF3F\uFF41-\uFF5A\uFF66-\uFFBE\uFFC2-\uFFC7\uFFCA-\uFFCF\uFFD2-\uFFD7\uFFDA-\uFFDC]|\uD800[\uDC00-\uDC0B\uDC0D-\uDC26\uDC28-\uDC3A\uDC3C\uDC3D\uDC3F-\uDC4D\uDC50-\uDC5D\uDC80-\uDCFA\uDD40-\uDD74\uDDFD\uDE80-\uDE9C\uDEA0-\uDED0\uDEE0\uDF00-\uDF1F\uDF2D-\uDF4A\uDF50-\uDF7A\uDF80-\uDF9D\uDFA0-\uDFC3\uDFC8-\uDFCF\uDFD1-\uDFD5]|\uD801[\uDC00-\uDC9D\uDCA0-\uDCA9\uDCB0-\uDCD3\uDCD8-\uDCFB\uDD00-\uDD27\uDD30-\uDD63\uDE00-\uDF36\uDF40-\uDF55\uDF60-\uDF67]|\uD802[\uDC00-\uDC05\uDC08\uDC0A-\uDC35\uDC37\uDC38\uDC3C\uDC3F-\uDC55\uDC60-\uDC76\uDC80-\uDC9E\uDCE0-\uDCF2\uDCF4\uDCF5\uDD00-\uDD15\uDD20-\uDD39\uDD80-\uDDB7\uDDBE\uDDBF\uDE00-\uDE03\uDE05\uDE06\uDE0C-\uDE13\uDE15-\uDE17\uDE19-\uDE33\uDE38-\uDE3A\uDE3F\uDE60-\uDE7C\uDE80-\uDE9C\uDEC0-\uDEC7\uDEC9-\uDEE6\uDF00-\uDF35\uDF40-\uDF55\uDF60-\uDF72\uDF80-\uDF91]|\uD803[\uDC00-\uDC48\uDC80-\uDCB2\uDCC0-\uDCF2]|\uD804[\uDC00-\uDC46\uDC66-\uDC6F\uDC7F-\uDCBA\uDCD0-\uDCE8\uDCF0-\uDCF9\uDD00-\uDD34\uDD36-\uDD3F\uDD50-\uDD73\uDD76\uDD80-\uDDC4\uDDCA-\uDDCC\uDDD0-\uDDDA\uDDDC\uDE00-\uDE11\uDE13-\uDE37\uDE3E\uDE80-\uDE86\uDE88\uDE8A-\uDE8D\uDE8F-\uDE9D\uDE9F-\uDEA8\uDEB0-\uDEEA\uDEF0-\uDEF9\uDF00-\uDF03\uDF05-\uDF0C\uDF0F\uDF10\uDF13-\uDF28\uDF2A-\uDF30\uDF32\uDF33\uDF35-\uDF39\uDF3C-\uDF44\uDF47\uDF48\uDF4B-\uDF4D\uDF50\uDF57\uDF5D-\uDF63\uDF66-\uDF6C\uDF70-\uDF74]|\uD805[\uDC00-\uDC4A\uDC50-\uDC59\uDC80-\uDCC5\uDCC7\uDCD0-\uDCD9\uDD80-\uDDB5\uDDB8-\uDDC0\uDDD8-\uDDDD\uDE00-\uDE40\uDE44\uDE50-\uDE59\uDE80-\uDEB7\uDEC0-\uDEC9\uDF00-\uDF19\uDF1D-\uDF2B\uDF30-\uDF39]|\uD806[\uDCA0-\uDCE9\uDCFF\uDE00-\uDE3E\uDE47\uDE50-\uDE83\uDE86-\uDE99\uDEC0-\uDEF8]|\uD807[\uDC00-\uDC08\uDC0A-\uDC36\uDC38-\uDC40\uDC50-\uDC59\uDC72-\uDC8F\uDC92-\uDCA7\uDCA9-\uDCB6\uDD00-\uDD06\uDD08\uDD09\uDD0B-\uDD36\uDD3A\uDD3C\uDD3D\uDD3F-\uDD47\uDD50-\uDD59]|\uD808[\uDC00-\uDF99]|\uD809[\uDC00-\uDC6E\uDC80-\uDD43]|[\uD80C\uD81C-\uD820\uD840-\uD868\uD86A-\uD86C\uD86F-\uD872\uD874-\uD879][\uDC00-\uDFFF]|\uD80D[\uDC00-\uDC2E]|\uD811[\uDC00-\uDE46]|\uD81A[\uDC00-\uDE38\uDE40-\uDE5E\uDE60-\uDE69\uDED0-\uDEED\uDEF0-\uDEF4\uDF00-\uDF36\uDF40-\uDF43\uDF50-\uDF59\uDF63-\uDF77\uDF7D-\uDF8F]|\uD81B[\uDF00-\uDF44\uDF50-\uDF7E\uDF8F-\uDF9F\uDFE0\uDFE1]|\uD821[\uDC00-\uDFEC]|\uD822[\uDC00-\uDEF2]|\uD82C[\uDC00-\uDD1E\uDD70-\uDEFB]|\uD82F[\uDC00-\uDC6A\uDC70-\uDC7C\uDC80-\uDC88\uDC90-\uDC99\uDC9D\uDC9E]|\uD834[\uDD65-\uDD69\uDD6D-\uDD72\uDD7B-\uDD82\uDD85-\uDD8B\uDDAA-\uDDAD\uDE42-\uDE44]|\uD835[\uDC00-\uDC54\uDC56-\uDC9C\uDC9E\uDC9F\uDCA2\uDCA5\uDCA6\uDCA9-\uDCAC\uDCAE-\uDCB9\uDCBB\uDCBD-\uDCC3\uDCC5-\uDD05\uDD07-\uDD0A\uDD0D-\uDD14\uDD16-\uDD1C\uDD1E-\uDD39\uDD3B-\uDD3E\uDD40-\uDD44\uDD46\uDD4A-\uDD50\uDD52-\uDEA5\uDEA8-\uDEC0\uDEC2-\uDEDA\uDEDC-\uDEFA\uDEFC-\uDF14\uDF16-\uDF34\uDF36-\uDF4E\uDF50-\uDF6E\uDF70-\uDF88\uDF8A-\uDFA8\uDFAA-\uDFC2\uDFC4-\uDFCB\uDFCE-\uDFFF]|\uD836[\uDE00-\uDE36\uDE3B-\uDE6C\uDE75\uDE84\uDE9B-\uDE9F\uDEA1-\uDEAF]|\uD838[\uDC00-\uDC06\uDC08-\uDC18\uDC1B-\uDC21\uDC23\uDC24\uDC26-\uDC2A]|\uD83A[\uDC00-\uDCC4\uDCD0-\uDCD6\uDD00-\uDD4A\uDD50-\uDD59]|\uD83B[\uDE00-\uDE03\uDE05-\uDE1F\uDE21\uDE22\uDE24\uDE27\uDE29-\uDE32\uDE34-\uDE37\uDE39\uDE3B\uDE42\uDE47\uDE49\uDE4B\uDE4D-\uDE4F\uDE51\uDE52\uDE54\uDE57\uDE59\uDE5B\uDE5D\uDE5F\uDE61\uDE62\uDE64\uDE67-\uDE6A\uDE6C-\uDE72\uDE74-\uDE77\uDE79-\uDE7C\uDE7E\uDE80-\uDE89\uDE8B-\uDE9B\uDEA1-\uDEA3\uDEA5-\uDEA9\uDEAB-\uDEBB]|\uD869[\uDC00-\uDED6\uDF00-\uDFFF]|\uD86D[\uDC00-\uDF34\uDF40-\uDFFF]|\uD86E[\uDC00-\uDC1D\uDC20-\uDFFF]|\uD873[\uDC00-\uDEA1\uDEB0-\uDFFF]|\uD87A[\uDC00-\uDFE0]|\uD87E[\uDC00-\uDE1D]|\uDB40[\uDD00-\uDDEF]/, 
Object.defineProperty(BU, "__esModule", {
    value: !0
}), BU.JudgeUtil = void 0;

const HU = UU;

BU.JudgeUtil = class {
    static isIgnoreChar(e) {
        return "string" == typeof e && ("\t" === e || "\v" === e || "\f" === e || " " === e || " " === e || "\ufeff" === e || "\n" === e || "\r" === e || "\u2028" === e || "\u2029" === e);
    }
    static isSpaceSeparator(e) {
        return "string" == typeof e && HU.Unicode.SPACE_SEPARATOR.test(e);
    }
    static isIdStartChar(e) {
        return "string" == typeof e && (e >= "a" && e <= "z" || e >= "A" && e <= "Z" || "$" === e || "_" === e || HU.Unicode.ID_START.test(e));
    }
    static isIdContinueChar(e) {
        return "string" == typeof e && (e >= "a" && e <= "z" || e >= "A" && e <= "Z" || e >= "0" && e <= "9" || "$" === e || "_" === e || "‌" === e || "‍" === e || HU.Unicode.ID_CONTINUE.test(e));
    }
    static isDigitWithoutZero(e) {
        return /[1-9]/.test(e);
    }
    static isDigit(e) {
        return "string" == typeof e && /[0-9]/.test(e);
    }
    static isHexDigit(e) {
        return "string" == typeof e && /[0-9A-Fa-f]/.test(e);
    }
};

var $U = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(NU, "__esModule", {
    value: !0
}), NU.parseJsonText = NU.parseJsonFile = void 0;

const GU = $U(t), WU = $U(a), VU = $U(r), KU = BU;

var qU;

!function(e) {
    e[e.Char = 0] = "Char", e[e.EOF = 1] = "EOF", e[e.Identifier = 2] = "Identifier";
}(qU || (qU = {}));

let JU, XU, ZU, YU, QU, ez, tz = "start", rz = [], nz = 0, oz = 1, iz = 0, az = !1, sz = "default", uz = "'", lz = 1;

function cz(e, t = !1) {
    XU = String(e), tz = "start", rz = [], nz = 0, oz = 1, iz = 0, YU = void 0, az = t;
    do {
        JU = fz(), yz[tz]();
    } while ("eof" !== JU.type);
    return YU;
}

function fz() {
    for (sz = "default", QU = "", uz = "'", lz = 1; ;) {
        ez = dz();
        const e = hz[sz]();
        if (e) {
            return e;
        }
    }
}

function dz() {
    if (XU[nz]) {
        return String.fromCodePoint(XU.codePointAt(nz));
    }
}

function pz() {
    const e = dz();
    return "\n" === e ? (oz++, iz = 0) : e ? iz += e.length : iz++, e && (nz += e.length), 
    e;
}

NU.parseJsonFile = function(e, t = !1, r = "utf-8") {
    const n = GU.default.readFileSync(VU.default.resolve(e), {
        encoding: r
    });
    try {
        return cz(n, t);
    } catch (t) {
        if (t instanceof SyntaxError) {
            const r = t.message.split("at");
            if (2 === r.length) {
                throw new Error(`${r[0].trim()}${WU.default.EOL}\t at ${e}:${r[1].trim()}`);
            }
        }
        throw new Error(`${e} is not in valid JSON/JSON5 format.`);
    }
}, NU.parseJsonText = cz;

const hz = {
    default() {
        switch (ez) {
          case "/":
            return pz(), void (sz = "comment");

          case void 0:
            return pz(), vz("eof");
        }
        if (!KU.JudgeUtil.isIgnoreChar(ez) && !KU.JudgeUtil.isSpaceSeparator(ez)) {
            return hz[tz]();
        }
        pz();
    },
    start() {
        sz = "value";
    },
    beforePropertyName() {
        switch (ez) {
          case "$":
          case "_":
            return QU = pz(), void (sz = "identifierName");

          case "\\":
            return pz(), void (sz = "identifierNameStartEscape");

          case "}":
            return vz("punctuator", pz());

          case '"':
          case "'":
            return uz = ez, pz(), void (sz = "string");
        }
        if (KU.JudgeUtil.isIdStartChar(ez)) {
            return QU += pz(), void (sz = "identifierName");
        }
        throw wz(qU.Char, pz());
    },
    afterPropertyName() {
        if (":" === ez) {
            return vz("punctuator", pz());
        }
        throw wz(qU.Char, pz());
    },
    beforePropertyValue() {
        sz = "value";
    },
    afterPropertyValue() {
        switch (ez) {
          case ",":
          case "}":
            return vz("punctuator", pz());
        }
        throw wz(qU.Char, pz());
    },
    beforeArrayValue() {
        if ("]" === ez) {
            return vz("punctuator", pz());
        }
        sz = "value";
    },
    afterArrayValue() {
        switch (ez) {
          case ",":
          case "]":
            return vz("punctuator", pz());
        }
        throw wz(qU.Char, pz());
    },
    end() {
        throw wz(qU.Char, pz());
    },
    comment() {
        switch (ez) {
          case "*":
            return pz(), void (sz = "multiLineComment");

          case "/":
            return pz(), void (sz = "singleLineComment");
        }
        throw wz(qU.Char, pz());
    },
    multiLineComment() {
        switch (ez) {
          case "*":
            return pz(), void (sz = "multiLineCommentAsterisk");

          case void 0:
            throw wz(qU.Char, pz());
        }
        pz();
    },
    multiLineCommentAsterisk() {
        switch (ez) {
          case "*":
            return void pz();

          case "/":
            return pz(), void (sz = "default");

          case void 0:
            throw wz(qU.Char, pz());
        }
        pz(), sz = "multiLineComment";
    },
    singleLineComment() {
        switch (ez) {
          case "\n":
          case "\r":
          case "\u2028":
          case "\u2029":
            return pz(), void (sz = "default");

          case void 0:
            return pz(), vz("eof");
        }
        pz();
    },
    value() {
        switch (ez) {
          case "{":
          case "[":
            return vz("punctuator", pz());

          case "n":
            return pz(), gz("ull"), vz("null", null);

          case "t":
            return pz(), gz("rue"), vz("boolean", !0);

          case "f":
            return pz(), gz("alse"), vz("boolean", !1);

          case "-":
          case "+":
            return "-" === pz() && (lz = -1), void (sz = "numerical");

          case ".":
          case "0":
          case "I":
          case "N":
            return void (sz = "numerical");

          case '"':
          case "'":
            return uz = ez, pz(), QU = "", void (sz = "string");
        }
        if (void 0 === ez || !KU.JudgeUtil.isDigitWithoutZero(ez)) {
            throw wz(qU.Char, pz());
        }
        sz = "numerical";
    },
    numerical() {
        switch (ez) {
          case ".":
            return QU = pz(), void (sz = "decimalPointLeading");

          case "0":
            return QU = pz(), void (sz = "zero");

          case "I":
            return pz(), gz("nfinity"), vz("numeric", lz * (1 / 0));

          case "N":
            return pz(), gz("aN"), vz("numeric", NaN);
        }
        if (void 0 !== ez && KU.JudgeUtil.isDigitWithoutZero(ez)) {
            return QU = pz(), void (sz = "decimalInteger");
        }
        throw wz(qU.Char, pz());
    },
    zero() {
        switch (ez) {
          case ".":
          case "e":
          case "E":
            return void (sz = "decimal");

          case "x":
          case "X":
            return QU += pz(), void (sz = "hexadecimal");
        }
        return vz("numeric", 0);
    },
    decimalInteger() {
        switch (ez) {
          case ".":
          case "e":
          case "E":
            return void (sz = "decimal");
        }
        if (!KU.JudgeUtil.isDigit(ez)) {
            return vz("numeric", lz * Number(QU));
        }
        QU += pz();
    },
    decimal() {
        switch (ez) {
          case ".":
            QU += pz(), sz = "decimalFraction";
            break;

          case "e":
          case "E":
            QU += pz(), sz = "decimalExponent";
        }
    },
    decimalPointLeading() {
        if (KU.JudgeUtil.isDigit(ez)) {
            return QU += pz(), void (sz = "decimalFraction");
        }
        throw wz(qU.Char, pz());
    },
    decimalFraction() {
        switch (ez) {
          case "e":
          case "E":
            return QU += pz(), void (sz = "decimalExponent");
        }
        if (!KU.JudgeUtil.isDigit(ez)) {
            return vz("numeric", lz * Number(QU));
        }
        QU += pz();
    },
    decimalExponent() {
        switch (ez) {
          case "+":
          case "-":
            return QU += pz(), void (sz = "decimalExponentSign");
        }
        if (KU.JudgeUtil.isDigit(ez)) {
            return QU += pz(), void (sz = "decimalExponentInteger");
        }
        throw wz(qU.Char, pz());
    },
    decimalExponentSign() {
        if (KU.JudgeUtil.isDigit(ez)) {
            return QU += pz(), void (sz = "decimalExponentInteger");
        }
        throw wz(qU.Char, pz());
    },
    decimalExponentInteger() {
        if (!KU.JudgeUtil.isDigit(ez)) {
            return vz("numeric", lz * Number(QU));
        }
        QU += pz();
    },
    hexadecimal() {
        if (KU.JudgeUtil.isHexDigit(ez)) {
            return QU += pz(), void (sz = "hexadecimalInteger");
        }
        throw wz(qU.Char, pz());
    },
    hexadecimalInteger() {
        if (!KU.JudgeUtil.isHexDigit(ez)) {
            return vz("numeric", lz * Number(QU));
        }
        QU += pz();
    },
    identifierNameStartEscape() {
        if ("u" !== ez) {
            throw wz(qU.Char, pz());
        }
        pz();
        const e = mz();
        switch (e) {
          case "$":
          case "_":
            break;

          default:
            if (!KU.JudgeUtil.isIdStartChar(e)) {
                throw wz(qU.Identifier);
            }
        }
        QU += e, sz = "identifierName";
    },
    identifierName() {
        switch (ez) {
          case "$":
          case "_":
          case "‌":
          case "‍":
            return void (QU += pz());

          case "\\":
            return pz(), void (sz = "identifierNameEscape");
        }
        if (!KU.JudgeUtil.isIdContinueChar(ez)) {
            return vz("identifier", QU);
        }
        QU += pz();
    },
    identifierNameEscape() {
        if ("u" !== ez) {
            throw wz(qU.Char, pz());
        }
        pz();
        const e = mz();
        switch (e) {
          case "$":
          case "_":
          case "‌":
          case "‍":
            break;

          default:
            if (!KU.JudgeUtil.isIdContinueChar(e)) {
                throw wz(qU.Identifier);
            }
        }
        QU += e, sz = "identifierName";
    },
    string() {
        switch (ez) {
          case "\\":
            return pz(), void (QU += function() {
                const e = dz(), t = function() {
                    switch (dz()) {
                      case "b":
                        return pz(), "\b";

                      case "f":
                        return pz(), "\f";

                      case "n":
                        return pz(), "\n";

                      case "r":
                        return pz(), "\r";

                      case "t":
                        return pz(), "\t";

                      case "v":
                        return pz(), "\v";
                    }
                    return;
                }();
                if (t) {
                    return t;
                }
                switch (e) {
                  case "0":
                    if (pz(), KU.JudgeUtil.isDigit(dz())) {
                        throw wz(qU.Char, pz());
                    }
                    return "\0";

                  case "x":
                    return pz(), function() {
                        let e = "", t = dz();
                        if (!KU.JudgeUtil.isHexDigit(t)) {
                            throw wz(qU.Char, pz());
                        }
                        if (e += pz(), t = dz(), !KU.JudgeUtil.isHexDigit(t)) {
                            throw wz(qU.Char, pz());
                        }
                        return e += pz(), String.fromCodePoint(parseInt(e, 16));
                    }();

                  case "u":
                    return pz(), mz();

                  case "\n":
                  case "\u2028":
                  case "\u2029":
                    return pz(), "";

                  case "\r":
                    return pz(), "\n" === dz() && pz(), "";
                }
                if (void 0 === e || KU.JudgeUtil.isDigitWithoutZero(e)) {
                    throw wz(qU.Char, pz());
                }
                return pz();
            }());

          case '"':
          case "'":
            if (ez === uz) {
                const e = vz("string", QU);
                return pz(), e;
            }
            return void (QU += pz());

          case "\n":
          case "\r":
          case void 0:
            throw wz(qU.Char, pz());

          case "\u2028":
          case "\u2029":
            !function(e) {
                console.warn(`JSON5: '${bz(e)}' in strings is not valid ECMAScript; consider escaping.`);
            }(ez);
        }
        QU += pz();
    }
};

function vz(e, t) {
    return {
        type: e,
        value: t,
        line: oz,
        column: iz
    };
}

function gz(e) {
    for (const t of e) {
        if (dz() !== t) {
            throw wz(qU.Char, pz());
        }
        pz();
    }
}

function mz() {
    let e = "", t = 4;
    for (;t-- > 0; ) {
        const t = dz();
        if (!KU.JudgeUtil.isHexDigit(t)) {
            throw wz(qU.Char, pz());
        }
        e += pz();
    }
    return String.fromCodePoint(parseInt(e, 16));
}

const yz = {
    start() {
        if ("eof" === JU.type) {
            throw wz(qU.EOF);
        }
        _z();
    },
    beforePropertyName() {
        switch (JU.type) {
          case "identifier":
          case "string":
            return ZU = JU.value, void (tz = "afterPropertyName");

          case "punctuator":
            return void Ez();

          case "eof":
            throw wz(qU.EOF);
        }
    },
    afterPropertyName() {
        if ("eof" === JU.type) {
            throw wz(qU.EOF);
        }
        tz = "beforePropertyValue";
    },
    beforePropertyValue() {
        if ("eof" === JU.type) {
            throw wz(qU.EOF);
        }
        _z();
    },
    afterPropertyValue() {
        if ("eof" === JU.type) {
            throw wz(qU.EOF);
        }
        switch (JU.value) {
          case ",":
            return void (tz = "beforePropertyName");

          case "}":
            Ez();
        }
    },
    beforeArrayValue() {
        if ("eof" === JU.type) {
            throw wz(qU.EOF);
        }
        "punctuator" !== JU.type || "]" !== JU.value ? _z() : Ez();
    },
    afterArrayValue() {
        if ("eof" === JU.type) {
            throw wz(qU.EOF);
        }
        switch (JU.value) {
          case ",":
            return void (tz = "beforeArrayValue");

          case "]":
            Ez();
        }
    },
    end() {}
};

function _z() {
    const e = function() {
        let e;
        switch (JU.type) {
          case "punctuator":
            switch (JU.value) {
              case "{":
                e = {};
                break;

              case "[":
                e = [];
            }
            break;

          case "null":
          case "boolean":
          case "numeric":
          case "string":
            e = JU.value;
        }
        return e;
    }();
    if (az && "object" == typeof e && (e._line = oz, e._column = iz), void 0 === YU) {
        YU = e;
    } else {
        const t = rz[rz.length - 1];
        Array.isArray(t) ? az && "object" != typeof e ? t.push({
            value: e,
            _line: oz,
            _column: iz
        }) : t.push(e) : t[ZU] = az && "object" != typeof e ? {
            value: e,
            _line: oz,
            _column: iz
        } : e;
    }
    !function(e) {
        if (e && "object" == typeof e) {
            rz.push(e), tz = Array.isArray(e) ? "beforeArrayValue" : "beforePropertyName";
        } else {
            const e = rz[rz.length - 1];
            tz = e ? Array.isArray(e) ? "afterArrayValue" : "afterPropertyValue" : "end";
        }
    }(e);
}

function Ez() {
    rz.pop();
    const e = rz[rz.length - 1];
    tz = e ? Array.isArray(e) ? "afterArrayValue" : "afterPropertyValue" : "end";
}

function bz(e) {
    const t = {
        "'": "\\'",
        '"': '\\"',
        "\\": "\\\\",
        "\b": "\\b",
        "\f": "\\f",
        "\n": "\\n",
        "\r": "\\r",
        "\t": "\\t",
        "\v": "\\v",
        "\0": "\\0",
        "\u2028": "\\u2028",
        "\u2029": "\\u2029"
    };
    if (t[e]) {
        return t[e];
    }
    if (e < " ") {
        const t = e.charCodeAt(0).toString(16);
        return `\\x${`00${t}`.substring(t.length)}`;
    }
    return e;
}

function wz(e, t) {
    let r = "";
    switch (e) {
      case qU.Char:
        r = void 0 === t ? `JSON5: invalid end of input at ${oz}:${iz}` : `JSON5: invalid character '${bz(t)}' at ${oz}:${iz}`;
        break;

      case qU.EOF:
        r = `JSON5: invalid end of input at ${oz}:${iz}`;
        break;

      case qU.Identifier:
        iz -= 5, r = `JSON5: invalid identifier character at ${oz}:${iz}`;
    }
    const n = new Dz(r);
    return n.lineNumber = oz, n.columnNumber = iz, n;
}

class Dz extends SyntaxError {}

var Sz = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(jU, "__esModule", {
    value: !0
}), jU.HvigorConfigLoader = void 0;

const Az = Sz(t), Oz = Sz(r), Cz = Sz(n), xz = LU, Fz = CU, Mz = FU, Pz = NU;

class Iz {
    constructor() {
        this.configPropertyMap = new Map;
        const e = Oz.default.resolve(Mz.HVIGOR_PROJECT_WRAPPER_HOME, Fz.DEFAULT_HVIGOR_CONFIG_JSON_FILE_NAME);
        if (!Az.default.existsSync(e)) {
            return;
        }
        const t = (0, Pz.parseJsonFile)(e), r = Oz.default.resolve(Mz.HVIGOR_USER_HOME, Fz.DEFAULT_HVIGOR_CONFIG_JSON_FILE_NAME);
        let n;
        Az.default.existsSync(r) && (n = (0, Pz.parseJsonFile)(r), t.properties = {
            ...n.properties,
            ...t.properties
        }), this.hvigorConfig = t;
    }
    static init(e) {
        var t, r;
        if (void 0 === e) {
            return void (Cz.default.env.config = void 0);
        }
        const n = Iz.getConfigs();
        let o = {};
        null === (t = e.config) || void 0 === t || t.forEach(e => {
            const t = e.split("=");
            2 === t.length && (o[t[0]] = t[t.length - 1], this.initCommandLineProperties(t[0], t[t.length - 1]));
        }), Array.isArray(e.prop) && (null === (r = e.prop) || void 0 === r || r.forEach(e => {
            const t = e.split("=");
            2 === t.length && (o[t[0]] = t[t.length - 1], this.initCommandLineProperties(t[0], t[t.length - 1]));
        })), o = {
            ...n,
            ...o
        }, Cz.default.env.config = JSON.stringify(o);
    }
    static initCommandLineProperties(e, t) {
        if (!e.startsWith(`${Fz.PROPERTIES + Fz.DOT}`)) {
            return;
        }
        const r = e.substring(`${Fz.PROPERTIES + Fz.DOT}`.length);
        xz.coreParameter.properties[r] = this.convertToParamValue(t);
    }
    static convertToParamValue(e) {
        let t = Number(e);
        return e.length <= 16 && !isNaN(t) ? t : (t = "true" === e || "false" !== e && t, 
        "boolean" == typeof t ? t : e.trim());
    }
    getHvigorConfig() {
        return this.hvigorConfig;
    }
    getPropertiesConfigValue(e) {
        var t;
        if (this.configPropertyMap.has(e)) {
            return this.configPropertyMap.get(e);
        }
        const r = Iz.getConfigs()["properties.".concat(e)], n = void 0 !== Cz.default.env.config && null !== (t = JSON.parse(Cz.default.env.config)["properties.".concat(e)]) && void 0 !== t ? t : r;
        if (void 0 !== n) {
            const t = this.parseConfigValue(n);
            return this.configPropertyMap.set(e, t), t;
        }
        if (void 0 === this.hvigorConfig) {
            return void this.configPropertyMap.set(e, void 0);
        }
        const o = this.hvigorConfig.properties ? this.hvigorConfig.properties[e] : void 0;
        return this.configPropertyMap.set(e, o), o;
    }
    static clean() {
        Iz.instance && (Iz.instance = void 0), Iz.config && (Iz.config = void 0);
    }
    static getInstance() {
        return Iz.instance || (Iz.instance = new Iz), Iz.instance;
    }
    static getConfigs() {
        if (this.config) {
            return this.config;
        }
        const e = Cz.default.argv.slice(2), t = /^(--config|-c).*/, r = /^(--config|-c)$/, n = {};
        for (const [o, i] of e.entries()) {
            if (r.test(i)) {
                if (o + 1 < e.length) {
                    const t = e[o + 1].split("=");
                    this.setConfig(t, n);
                }
            } else if (t.test(i)) {
                const e = i.match(t);
                if (e && e[0].length < i.length) {
                    const t = i.substring(e[0].length).split("=");
                    this.setConfig(t, n);
                }
            }
        }
        return this.config = n, n;
    }
    static setConfig(e, t) {
        2 === e.length && (t[e[0]] = e[e.length - 1]);
    }
    parseConfigValue(e) {
        if ("true" === e.toLowerCase()) {
            return !0;
        }
        if ("false" === e.toLowerCase()) {
            return !1;
        }
        const t = Number(e);
        return isNaN(t) ? e : t;
    }
}

jU.HvigorConfigLoader = Iz, Iz.config = void 0;

var kz = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(SU, "__esModule", {
    value: !0
}), SU.PathUtil = void 0;

const Rz = kz(Bs), Tz = kz(r), jz = kz(n), Lz = AU, Nz = CU, Bz = FU, Uz = jU;

class zz {
    static getHvigorCacheDir(e) {
        var t;
        let r = void 0 !== jz.default.env.config ? JSON.parse(jz.default.env.config)[Nz.BUILD_CACHE_DIR] : null !== (t = (0, 
        Lz.getExtraConfig)(Nz.BUILD_CACHE_DIR)) && void 0 !== t ? t : zz.getCommandHvigorCacheDir();
        const n = Tz.default.resolve(Bz.HVIGOR_PROJECT_ROOT_DIR, Nz.HVIGOR_USER_HOME_DIR_NAME);
        return r || (r = Uz.HvigorConfigLoader.getInstance().getPropertiesConfigValue(Nz.HVIGOR_CACHE_DIR_KEY), 
        r) ? Tz.default.isAbsolute(r) ? (e && !this.hvigorCacheDirHasLogged && (e.warn("Please ensure no projects of the same name have the same custom hvigor data dir."), 
        this.hvigorCacheDirHasLogged = !0), Tz.default.resolve(r, Tz.default.basename(jz.default.cwd()), Nz.HVIGOR_USER_HOME_DIR_NAME)) : (e && !this.hvigorCacheDirHasLogged && (e.warn(`Invalid custom hvigor data dir:${r}`), 
        this.hvigorCacheDirHasLogged = !0), n) : n;
    }
    static checkCopyPathIsSame(e, t) {
        const r = zz.getStatsSync(e), n = zz.getStatsSync(t);
        return !(!n || !r) && !!zz.areIdentical(r, n);
    }
    static getStatsSync(e) {
        let t;
        try {
            t = Rz.default.statSync(e);
        } catch (e) {
            return null;
        }
        return t;
    }
    static areIdentical(e, t) {
        return t.ino && t.dev && t.ino === e.ino && t.dev === e.dev;
    }
    static getCommandHvigorCacheDir() {
        return jz.default.argv.forEach(e => {
            e.startsWith(Nz.BUILD_CACHE_DIR) && (jz.default.env.BUILD_CACHE_DIR = e.substring(e.indexOf("=") + 1));
        }), jz.default.env.BUILD_CACHE_DIR;
    }
    static getReportDirPath() {
        return Tz.default.resolve(zz.getHvigorCacheDir(), "report");
    }
}

SU.PathUtil = zz, zz.hvigorCacheDirHasLogged = !1;

var Hz = {};

Object.defineProperty(Hz, "__esModule", {
    value: !0
}), Hz.replacer = void 0, Hz.replacer = function(e, t) {
    if (t instanceof Map) {
        const e = Object.create(null);
        return t.forEach((t, r) => {
            e[r] = t;
        }), e;
    }
    if (t instanceof Set) {
        const e = [];
        return t.forEach(t => {
            e.push(t);
        }), e;
    }
    return t;
}, function(e) {
    var t = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.traceManager = e.TraceManager = void 0;
    const n = t(r), o = Uy, i = t(Bs), a = y, s = SU, u = Hz;
    class l {
        constructor() {
            this.callBackList = {}, this.data = {}, this.callBackList = {};
        }
        trace(e, t, r) {
            this.data[e] = t, "function" == typeof r && (this.callBackList[e] = r);
        }
        flush() {
            const e = (0, o.cloneDeep)(this.data);
            for (const e in this.callBackList) {
                this.callBackList[e]();
            }
            const t = n.default.resolve(s.PathUtil.getHvigorCacheDir(), "./outputs/logs/details"), r = n.default.resolve(t, "details.json");
            i.default.ensureDirSync(t);
            try {
                i.default.writeFileSync(r, JSON.stringify(e, u.replacer, 2));
            } catch (e) {}
        }
        static transformKey(e) {
            return e.replace(/\./g, "_");
        }
        static anonymize(e) {
            return (0, a.hash)(e);
        }
        static trace(t, r, n) {
            e.traceManager.trace(t, r, n);
        }
    }
    e.TraceManager = l, e.traceManager = new l;
}(By), Object.defineProperty(Ny, "__esModule", {
    value: !0
}), Ny.hvigorTrace = void 0;

const $z = By;

class Gz {
    constructor() {
        this.configBlacklist = new Set([ "hvigor.cacheDir", "ohos.buildDir", "ohos.arkCompile.sourceMapDir" ]), 
        this.data = {
            IS_INCREMENTAL: !0,
            IS_DAEMON: !0,
            IS_PARALLEL: !0,
            IS_HVIGORFILE_TYPE_CHECK: !1,
            TASK_TIME: {},
            APIS: new Set
        };
    }
    transmitDataToManager() {
        $z.TraceManager.trace(Gz.TRACE_KEY, this.data, () => {
            delete this.data.BUILD_ID, delete this.data.ERROR_MESSAGE, this.data.TASK_TIME = {}, 
            this.data.APIS.clear();
        });
    }
    traceTotalTime(e) {
        this.data.TOTAL_TIME = e;
    }
    traceBaseConfig(e, t, r, n) {
        this.data.IS_INCREMENTAL = e, this.data.IS_DAEMON = t, this.data.IS_PARALLEL = r, 
        this.data.IS_HVIGORFILE_TYPE_CHECK = n;
    }
    traceBuildId(e) {
        this.data.BUILD_ID = e;
    }
    traceTaskTime(e, t, r) {
        var n, o;
        let i;
        i = "" === t ? "APP" : $z.TraceManager.transformKey($z.TraceManager.anonymize(t));
        const a = e.substring(e.indexOf("@") + 1), s = null !== (o = null === (n = this.data.TASK_TIME) || void 0 === n ? void 0 : n[i]) && void 0 !== o ? o : {};
        s[a] = r, this.data.TASK_TIME && (this.data.TASK_TIME[i] = s);
    }
    traceErrorMessage(e) {
        var t, r;
        this.data.ERROR_MESSAGE = null !== (t = this.data.ERROR_MESSAGE) && void 0 !== t ? t : [], 
        this.data.ERROR_MESSAGE.push({
            CODE: e.code,
            MESSAGE: e.originMessage,
            SOLUTIONS: e.originSolutions,
            MORE_INFO: e.moreInfo,
            TIMESTAMP: null === (r = e.timestamp) || void 0 === r ? void 0 : r.getTime().toString(),
            COMPONENTS: e.components,
            CHECK_MESSAGE: e.checkMessage
        });
    }
    insertUsedApi(e) {
        this.data.APIS.has(e) || this.data.APIS.add(e);
    }
    traceConfigProperties(e) {
        e = Object.entries(e).reduce((e, [t, r]) => {
            const n = $z.TraceManager.transformKey(t);
            return r = this.configBlacklist.has(t) ? "" : r, e[n] = r, e;
        }, {}), this.data.CONFIG_PROPERTIES = e;
    }
}

Gz.TRACE_KEY = "HVIGOR", Ny.hvigorTrace = new Gz;

var Wz = {}, Vz = {};

!function(e) {
    var t;
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.BaseEvent = e.EventBody = e.EventHead = e.MetricEventType = void 0, (t = e.MetricEventType || (e.MetricEventType = {})).DURATION = "duration", 
    t.INSTANT = "instant", t.COUNTER = "counter", t.GAUGE = "gauge", t.OBJECT = "object", 
    t.METADATA = "metadata", t.MARK = "mark", t.LOG = "log", t.CONTINUAL = "continual";
    e.EventHead = class {
        constructor(e, t, r, n) {
            this.id = e, this.name = t, this.description = r, this.type = n;
        }
    };
    e.EventBody = class {
        constructor(e, t) {
            this.pid = e, this.tid = t, this.startTime = Number(process.hrtime.bigint());
        }
    };
    e.BaseEvent = class {
        constructor(e, t) {
            this.head = e, this.body = t, this.additional = {};
        }
        setStartTime(e) {
            this.body.startTime = null != e ? e : Number(process.hrtime.bigint());
        }
        setEndTime(e) {
            this.body.endTime = null != e ? e : Number(process.hrtime.bigint());
        }
        setTotalTime(e) {
            this.body.totalTime = e;
        }
        getId() {
            return this.head.id;
        }
        getName() {
            return this.head.name;
        }
        getDescription() {
            return this.head.description;
        }
        setName(e) {
            this.head.name = e;
        }
        getType() {
            return this.head.type;
        }
        setType(e) {
            this.head.type = e;
        }
        getTid() {
            return this.body.tid;
        }
        setTid(e) {
            return this.body.tid = e, this;
        }
    };
}(Vz), function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.LogEvent = e.LogEventAdditional = e.MetricLogType = void 0;
    const t = Vz;
    var r;
    (r = e.MetricLogType || (e.MetricLogType = {})).DEBUG = "debug", r.INFO = "info", 
    r.WARN = "warn", r.ERROR = "error", r.DETAIL = "detail";
    class n {
        constructor(e) {
            this.logType = e, this.children = [];
        }
    }
    e.LogEventAdditional = n;
    class o extends t.BaseEvent {
        constructor(e, r, o, i, a, s) {
            super(new t.EventHead(e, r, o, t.MetricEventType.LOG), new t.EventBody(i, a)), this.additional = new n(s);
        }
        getLogType() {
            return this.additional.logType;
        }
        setLogType(e) {
            this.additional.logType = e;
        }
        getDurationId() {
            return this.additional.durationId;
        }
        setDurationId(e) {
            this.additional.durationId = e;
        }
        getContinualId() {
            return this.additional.continualId;
        }
        setContinualId(e) {
            this.additional.continualId = e;
        }
        addChild(e) {
            e && -1 === this.additional.children.indexOf(e) && this.additional.children.push(e);
        }
        setParent(e) {
            this.additional.parent || (this.additional.parent = e);
        }
    }
    e.LogEvent = o;
}(Wz);

var Kz = {}, qz = {}, Jz = {}, Xz = {};

Object.defineProperty(Xz, "__esModule", {
    value: !0
}), Xz.Report = void 0;

Xz.Report = class {
    constructor(e, t) {
        this.name = e, this.value = t;
    }
    getName() {
        return this.name;
    }
    getValue() {
        return this.value;
    }
};

var Zz = {}, Yz = {}, Qz = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(Yz, "__esModule", {
    value: !0
}), Yz.LocalFileWriter = void 0;

const eH = Qz(r), tH = Qz(Bs), rH = Hz;

class nH {
    constructor() {
        this._replacer = rH.replacer, this._space = 2;
    }
    withSpace(e) {
        this._space = e;
    }
    withReplacer(e) {
        this._replacer = e;
    }
    write(e, t) {
        try {
            const r = JSON.stringify(t, this._replacer, this._space);
            this.writeStr(e, r);
        } catch (r) {
            const n = this.chunkStringify(t);
            this.writeStrArr(e, n);
        }
    }
    chunkStringify(e) {
        const t = Object.keys(e), r = [ "{\n" ], n = new Array(this._space).fill(" ").join("");
        return t.forEach(t => {
            if (Array.isArray(e[t]) && e[t].length) {
                r.push(`${n}${JSON.stringify(t)}: [\n`);
                let o = "";
                for (let i = 0; i < e[t].length; i++) {
                    const a = e[t][i], s = `${JSON.stringify(a, this._replacer, this._space).split("\n").map(e => `${n}${n}${e}`).join("\n")},\n`;
                    o.length >= 1e8 ? (r.push(o), o = s) : o += s;
                }
                r.push(`${o.replace(/,\n$/, "\n")}${n}],\n`);
            } else {
                r.push(`${n}${JSON.stringify(t)}: ${JSON.stringify(e[t], this._replacer, this._space)},\n`);
            }
        }), r[r.length - 1] = r[r.length - 1].replace(/,\n$/, "\n"), r.push("}"), r;
    }
    writeStr(e, t) {
        const r = eH.default.dirname(e);
        tH.default.existsSync(r) || tH.default.mkdirSync(r, {
            recursive: !0
        }), tH.default.writeFileSync(e, t);
    }
    writeStrArr(e, t) {
        const r = eH.default.dirname(e);
        tH.default.existsSync(r) || tH.default.mkdirSync(r, {
            recursive: !0
        }), tH.default.writeFileSync(e, ""), t.forEach(t => {
            tH.default.appendFileSync(e, t);
        });
    }
    static getInstance() {
        return nH.instance || (nH.instance = new nH), nH.instance;
    }
}

Yz.LocalFileWriter = nH;

var oH = {}, iH = {};

!function(e) {
    var t = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.levelMap = e.getLevel = e.setCategoriesLevel = e.updateConfiguration = e.getConfiguration = e.setConfiguration = e.logFilePath = void 0;
    const n = Gm, o = t(r), i = SU, a = FU, s = CU;
    e.logFilePath = () => {
        let e;
        try {
            e = i.PathUtil.getHvigorCacheDir();
        } catch {
            e = o.default.resolve(a.HVIGOR_PROJECT_ROOT_DIR, s.HVIGOR_USER_HOME_DIR_NAME);
        }
        return o.default.resolve(e, "./outputs/build-logs");
    };
    let u = {
        appenders: {
            debug: {
                type: "stdout",
                layout: {
                    type: "pattern",
                    pattern: "[%d] > hvigor %p %c %[%m%]"
                }
            },
            "debug-log-file": {
                type: "file",
                filename: o.default.resolve((0, e.logFilePath)(), "build.log"),
                maxLogSize: 2097152,
                backups: 9,
                encoding: "utf-8",
                level: "debug"
            },
            info: {
                type: "stdout",
                layout: {
                    type: "pattern",
                    pattern: "[%d] > hvigor %[%m%]"
                }
            },
            "no-pattern-info": {
                type: "stdout",
                layout: {
                    type: "pattern",
                    pattern: "%m"
                }
            },
            wrong: {
                type: "stderr",
                layout: {
                    type: "pattern",
                    pattern: "[%d] > hvigor %[%p: %m%]"
                }
            },
            "just-debug": {
                type: "logLevelFilter",
                appender: "debug",
                level: "debug",
                maxLevel: "debug"
            },
            "just-info": {
                type: "logLevelFilter",
                appender: "info",
                level: "info",
                maxLevel: "info"
            },
            "just-wrong": {
                type: "logLevelFilter",
                appender: "wrong",
                level: "warn",
                maxLevel: "error"
            }
        },
        categories: {
            default: {
                appenders: [ "just-debug", "just-info", "just-wrong" ],
                level: "debug"
            },
            "no-pattern-info": {
                appenders: [ "no-pattern-info" ],
                level: "info"
            },
            "debug-file": {
                appenders: [ "debug-log-file" ],
                level: "debug"
            }
        }
    };
    e.setConfiguration = e => {
        u = e;
    };
    e.getConfiguration = () => u;
    e.updateConfiguration = () => {
        const t = u.appenders["debug-log-file"];
        return t && "filename" in t && (t.filename = o.default.resolve((0, e.logFilePath)(), "build.log")), 
        u;
    };
    let l = n.levels.DEBUG;
    e.setCategoriesLevel = (e, t) => {
        l = e;
        const r = u.categories;
        for (const n in r) {
            (null == t ? void 0 : t.includes(n)) || n.includes("file") || Object.prototype.hasOwnProperty.call(r, n) && (r[n].level = e.levelStr);
        }
    };
    e.getLevel = () => l, e.levelMap = new Map([ [ "ALL", n.levels.ALL ], [ "MARK", n.levels.MARK ], [ "TRACE", n.levels.TRACE ], [ "DEBUG", n.levels.DEBUG ], [ "INFO", n.levels.INFO ], [ "WARN", n.levels.WARN ], [ "ERROR", n.levels.ERROR ], [ "FATAL", n.levels.FATAL ], [ "OFF", n.levels.OFF ], [ "all", n.levels.ALL ], [ "mark", n.levels.MARK ], [ "trace", n.levels.TRACE ], [ "debug", n.levels.DEBUG ], [ "info", n.levels.INFO ], [ "warn", n.levels.WARN ], [ "error", n.levels.ERROR ], [ "fatal", n.levels.FATAL ], [ "off", n.levels.OFF ] ]);
}(iH);

var aH, sH, uH, lH, cH = {}, fH = {};

function dH() {
    return sH || (sH = 1, function(e) {
        var o = g && g.__importDefault || function(e) {
            return e && e.__esModule ? e : {
                default: e
            };
        };
        Object.defineProperty(e, "__esModule", {
            value: !0
        }), e.HvigorConfigReader = e.defaultOptions = void 0;
        const i = o(t), a = o(r), s = o(n), u = CU, l = FU, c = jU, f = function() {
            if (aH) {
                return fH;
            }
            aH = 1;
            var e = g && g.__createBinding || (Object.create ? function(e, t, r, n) {
                void 0 === n && (n = r);
                var o = Object.getOwnPropertyDescriptor(t, r);
                o && !("get" in o ? !t.__esModule : o.writable || o.configurable) || (o = {
                    enumerable: !0,
                    get: function() {
                        return t[r];
                    }
                }), Object.defineProperty(e, n, o);
            } : function(e, t, r, n) {
                void 0 === n && (n = r), e[n] = t[r];
            }), n = g && g.__setModuleDefault || (Object.create ? function(e, t) {
                Object.defineProperty(e, "default", {
                    enumerable: !0,
                    value: t
                });
            } : function(e, t) {
                e.default = t;
            }), o = g && g.__importStar || function(t) {
                if (t && t.__esModule) {
                    return t;
                }
                var r = {};
                if (null != t) {
                    for (var o in t) {
                        "default" !== o && Object.prototype.hasOwnProperty.call(t, o) && e(r, t, o);
                    }
                }
                return n(r, t), r;
            };
            Object.defineProperty(fH, "__esModule", {
                value: !0
            }), fH.Json5Reader = void 0;
            const i = o(t), a = v, s = o(r), u = NU, l = WH();
            class c {
                static getJson5Obj(e, t = "utf-8") {
                    i.existsSync(e) || c.logger.printErrorExit("FILE_NOT_EXIST", [ e ]);
                    const r = i.readFileSync(s.resolve(e), {
                        encoding: t
                    });
                    try {
                        return (0, u.parseJsonText)(r);
                    } catch (t) {
                        c.handleException(e, t);
                    }
                }
                static async readJson5File(e, t = "utf-8") {
                    try {
                        return (0, a.readFile)(e, {
                            encoding: t
                        }).then(u.parseJsonText);
                    } catch (t) {
                        c.handleException(e, t);
                    }
                }
                static handleException(e, t) {
                    if (t instanceof SyntaxError) {
                        const r = t.message.split("at ");
                        2 === r.length && c.logger.printErrorExit("JSON_READER_SYNTAX_ERROR", [ r[0].trim(), e, r[1].trim() ]);
                    }
                    c.logger.printErrorExit("NOT_CORRECT_JSON_FORMAT", [ e ]);
                }
                static getJson5ObjProp(e, t) {
                    const r = t.split(".");
                    let n = e;
                    for (const e of r) {
                        if (void 0 === n[e]) {
                            return;
                        }
                        n = n[e];
                    }
                    return n;
                }
            }
            return fH.Json5Reader = c, c.logger = l.HvigorLogger.getLogger(c.name), fH;
        }();
        e.defaultOptions = {
            maxOldSpaceSize: 8192,
            maxSemiSpaceSize: 16,
            exposeGC: !0
        };
        class d extends f.Json5Reader {
            static getHvigorConfig() {
                const e = a.default.resolve(l.HVIGOR_PROJECT_WRAPPER_HOME, u.DEFAULT_HVIGOR_CONFIG_JSON_FILE_NAME);
                if (!i.default.existsSync(e)) {
                    return;
                }
                const t = this.getJson5Obj(e), r = a.default.resolve(l.HVIGOR_USER_HOME, u.DEFAULT_HVIGOR_CONFIG_JSON_FILE_NAME);
                let n;
                return i.default.existsSync(r) && (n = this.getJson5Obj(r), t.properties = {
                    ...n.properties,
                    ...t.properties
                }), t;
            }
            static getPropertiesConfigValue(e) {
                return c.HvigorConfigLoader.getInstance().getPropertiesConfigValue(e);
            }
            static getMaxOldSpaceSize(t = !1) {
                var r, n, o, i;
                const a = s.default.argv.find(e => e.startsWith(this.maxOldSpaceSizeParamPrefiex)), u = Number(null !== (r = null == a ? void 0 : a.slice((null == a ? void 0 : a.indexOf("=")) + 1)) && void 0 !== r ? r : ""), l = null === (o = null === (n = d.getHvigorConfig()) || void 0 === n ? void 0 : n.nodeOptions) || void 0 === o ? void 0 : o.maxOldSpaceSize, c = s.default.execArgv.find(e => e.startsWith(this.maxOldSpaceSizeParamPrefiex)), f = {
                    argv: u,
                    config: l,
                    execArgv: Number(null !== (i = null == c ? void 0 : c.slice((null == c ? void 0 : c.indexOf("=")) + 1)) && void 0 !== i ? i : ""),
                    default: e.defaultOptions.maxOldSpaceSize
                };
                return this.getPriorVal(f, t);
            }
            static getStacktrace(e = !1) {
                var t, r, n;
                const o = [ ...s.default.argv ].reverse().find(e => e === this.STACK_TRACE || e === this.NO_STACK_TRACE), i = o === this.STACK_TRACE || o !== this.NO_STACK_TRACE && void 0, a = null === (r = null === (t = d.getHvigorConfig()) || void 0 === t ? void 0 : t.debugging) || void 0 === r ? void 0 : r.stacktrace, u = !1;
                return e ? null != a ? a : u : null !== (n = null != i ? i : a) && void 0 !== n ? n : u;
            }
            static getMaxSemiSpaceSize(t = !1) {
                var r, n, o, i;
                const a = s.default.argv.find(e => e.startsWith(this.maxSemiSpaceSizeParamPrefiex)), u = Number(null !== (r = null == a ? void 0 : a.slice((null == a ? void 0 : a.indexOf("=")) + 1)) && void 0 !== r ? r : ""), l = null === (o = null === (n = d.getHvigorConfig()) || void 0 === n ? void 0 : n.nodeOptions) || void 0 === o ? void 0 : o.maxSemiSpaceSize, c = s.default.execArgv.find(e => e.startsWith(this.maxSemiSpaceSizeParamPrefiex)), f = {
                    argv: u,
                    config: l,
                    execArgv: Number(null !== (i = null == c ? void 0 : c.slice((null == c ? void 0 : c.indexOf("=")) + 1)) && void 0 !== i ? i : ""),
                    default: e.defaultOptions.maxSemiSpaceSize
                };
                return this.getPriorVal(f, t);
            }
            static getNodeParamFromProcessArgv() {
                const e = [];
                for (const t of [ this.maxOldSpaceSizeParamPrefiex, this.maxSemiSpaceSizeParamPrefiex ]) {
                    const r = s.default.argv.find(e => e.startsWith(t));
                    r && e.push(r);
                }
                return e;
            }
            static getPriorVal(e, t) {
                return t ? e.config || e.default : e.argv || e.config || e.execArgv || e.default;
            }
        }
        e.HvigorConfigReader = d, d.maxOldSpaceSizeParamPrefiex = "--max-old-space-size=", 
        d.maxSemiSpaceSizeParamPrefiex = "--max-semi-space-size=", d.STACK_TRACE = "--stacktrace", 
        d.NO_STACK_TRACE = "--no-stacktrace";
    }(cH)), cH;
}

function pH() {
    return uH || (uH = 1, function(e) {
        var t, r;
        Object.defineProperty(e, "__esModule", {
            value: !0
        }), e.resetStartData = e.refreshDaemonProcessEnv = e.initStartData = e.startEnvironment = e.defaultStartEnvironment = e.globalData = void 0;
        const n = Gm, o = CU, i = jU, a = Ny, s = iH, u = WH(), l = dH(), c = LU;
        e.globalData = new class {
            init(e, t) {
                this.cliEnv = e, this.cliOpts = t, this.buildId = function() {
                    const e = new Date, t = e.getFullYear(), r = `0${e.getMonth() + 1}`.slice(-2), n = `0${e.getDate()}`.slice(-2), o = `0${e.getHours()}`.slice(-2), i = `0${e.getMinutes()}`.slice(-2), a = `0${e.getSeconds()}`.slice(-2), s = `00${e.getMilliseconds()}`.slice(-3), u = `${t}${r}${n}${o}${i}${a}${s}`;
                    u !== g ? (g = u, m = 0) : m++;
                    return `${u}${m}`;
                }(), a.hvigorTrace.traceBuildId(this.buildId);
            }
            clean() {
                this.buildId = void 0;
            }
        };
        const f = {
            pageType: "page",
            product: "default",
            buildRoot: ".test",
            unitTestMode: "true",
            isLocalTest: "true",
            "ohos-test-coverage": "true",
            "unit.test.replace.page": "../../../.test/testability/pages/Index"
        }, d = {
            product: "default",
            buildMode: "test",
            isOhosTest: "true",
            "ohos-test-coverage": "true"
        };
        function p() {
            const e = i.HvigorConfigLoader.getInstance();
            c.coreParameter.properties.hvigorPoolMaxSize = h(e.getPropertiesConfigValue(o.HVIGOR_POOL_MAX_SIZE)), 
            c.coreParameter.properties.hvigorPoolMaxCoreSize = h(e.getPropertiesConfigValue(o.HVIGOR_POOL_MAX_CORE_SIZE)), 
            c.coreParameter.properties.hvigorPoolCacheCapacity = h(e.getPropertiesConfigValue(o.HVIGOR_POOL_CACHE_CAPACITY)), 
            c.coreParameter.properties.hvigorPoolCacheTtl = h(e.getPropertiesConfigValue(o.HVIGOR_POOL_CACHE_TTL)), 
            c.coreParameter.properties.ohosArkCompileMaxSize = h(e.getPropertiesConfigValue(o.OHOS_ARK_COMPILE_MAX_SIZE)), 
            c.coreParameter.properties.hvigorMemoryThreshold = h(e.getPropertiesConfigValue(o.HVIGOR_MEMORY_THRESHOLD));
        }
        function h(e) {
            if (!("string" == typeof e || void 0 === e || e < 0)) {
                return Math.floor(e);
            }
        }
        function v(e) {
            const t = new Map;
            if (!e) {
                return t;
            }
            return ("string" == typeof e ? [ e ] : e).forEach(e => {
                const [r, n] = e.split("="), o = "coverage" === r ? "ohos-test-coverage" : r;
                t.set(o, n);
            }), t;
        }
        e.defaultStartEnvironment = {
            nodeHome: null !== (t = process.env.NODE_HOME) && void 0 !== t ? t : "",
            workspaceDir: null !== (r = process.env.WORKSPACE_DIR) && void 0 !== r ? r : ""
        }, e.startEnvironment = {
            ...e.defaultStartEnvironment
        }, e.initStartData = function(t) {
            i.HvigorConfigLoader.init(t), u.HvigorLogger.clean(), function(t) {
                if (!t) {
                    return;
                }
                const r = new Map;
                void 0 !== t.prop && [ t.prop ].flat(2).forEach(e => {
                    var t;
                    const n = e.split("=");
                    r.set(n[0], null === (t = null == n ? void 0 : n.splice(1)) || void 0 === t ? void 0 : t.join("="));
                });
                c.coreParameter.extParams = Object.fromEntries(r.entries()), c.coreParameter.workspaceDir = e.startEnvironment.workspaceDir;
            }(t), p(), function() {
                const t = l.HvigorConfigReader.getHvigorConfig();
                if (!t) {
                    return void (c.coreParameter.properties = {
                        ...c.defaultProperties,
                        ...c.coreParameter.properties
                    });
                }
                e.startEnvironment = {
                    ...e.startEnvironment,
                    ...t.environment
                }, c.coreParameter.properties = {
                    ...c.defaultProperties,
                    ...t.properties,
                    ...c.coreParameter.properties
                }, function(e) {
                    var t, r, n, o, i, a, u, l, f, d, p, h;
                    c.coreParameter.startParams.incrementalExecution = null !== (r = null === (t = e.execution) || void 0 === t ? void 0 : t.incremental) && void 0 !== r ? r : c.coreParameter.startParams.incrementalExecution, 
                    c.coreParameter.startParams.hvigorfileTypeCheck = null !== (o = null === (n = e.execution) || void 0 === n ? void 0 : n.typeCheck) && void 0 !== o ? o : c.coreParameter.startParams.hvigorfileTypeCheck, 
                    c.coreParameter.startParams.parallelExecution = null !== (a = null === (i = e.execution) || void 0 === i ? void 0 : i.parallel) && void 0 !== a ? a : c.coreParameter.startParams.parallelExecution, 
                    c.coreParameter.startParams.daemon = null !== (l = null === (u = e.execution) || void 0 === u ? void 0 : u.daemon) && void 0 !== l ? l : c.coreParameter.startParams.daemon, 
                    c.coreParameter.startParams.printStackTrace = null !== (d = null === (f = e.debugging) || void 0 === f ? void 0 : f.stacktrace) && void 0 !== d ? d : c.coreParameter.startParams.printStackTrace, 
                    c.coreParameter.startParams.optimizationStrategy = null !== (h = null === (p = e.execution) || void 0 === p ? void 0 : p.optimizationStrategy) && void 0 !== h ? h : c.coreParameter.startParams.optimizationStrategy, 
                    function(e) {
                        var t, r, n;
                        (null === (t = e.logging) || void 0 === t ? void 0 : t.level) && (c.coreParameter.startParams.logLevel = null !== (n = s.levelMap.get(null === (r = e.logging) || void 0 === r ? void 0 : r.level)) && void 0 !== n ? n : c.coreParameter.startParams.logLevel);
                    }(e);
                }(t);
            }(), function(e) {
                if (!e) {
                    return;
                }
                const t = e._.includes("test");
                if (!t) {
                    return;
                }
                e.mode || (e.mode = "module");
                const r = v(e.prop);
                Object.keys(f).forEach(e => {
                    r.has(e) || r.set(e, f[e]);
                });
                const n = [];
                r.forEach((e, t) => {
                    n.push(`${t}=${e}`);
                }), e.prop = n;
            }(t), function(e) {
                if (!e) {
                    return;
                }
                const t = e._.includes("onDeviceTest");
                if (!t) {
                    return;
                }
                e.mode || (e.mode = "module");
                const r = v(e.prop);
                Object.keys(d).forEach(e => {
                    r.has(e) || r.set(e, d[e]);
                });
                const n = [];
                r.forEach((e, t) => {
                    n.push(`${t}=${e}`);
                }), e.prop = n;
            }(t), function(t) {
                var r, o, i, s, u, l, f;
                const d = null != t ? t : e.globalData.cliOpts;
                e.startEnvironment.nodeHome = null !== (r = d.nodeHome) && void 0 !== r ? r : e.startEnvironment.nodeHome, 
                c.coreParameter.startParams.hotCompile = d.hotCompile, c.coreParameter.startParams.hotReloadBuild = d.hotReloadBuild, 
                c.coreParameter.startParams.hvigorfileTypeCheck = null !== (o = d.enableBuildScriptTypeCheck) && void 0 !== o ? o : c.coreParameter.startParams.hvigorfileTypeCheck, 
                c.coreParameter.startParams.hvigorfileTypeCheck = null !== (i = d.typeCheck) && void 0 !== i ? i : c.coreParameter.startParams.hvigorfileTypeCheck, 
                c.coreParameter.startParams.daemon = null !== (s = d.daemon) && void 0 !== s ? s : c.coreParameter.startParams.daemon, 
                c.coreParameter.startParams.printStackTrace = null !== (u = d.stacktrace) && void 0 !== u ? u : c.coreParameter.startParams.printStackTrace, 
                c.coreParameter.startParams.logLevel = d.debug ? n.levels.DEBUG : d.warn ? n.levels.WARN : d.error ? n.levels.ERROR : d.info ? n.levels.INFO : c.coreParameter.startParams.logLevel, 
                c.coreParameter.startParams.parallelExecution = null !== (l = d.parallel) && void 0 !== l ? l : c.coreParameter.startParams.parallelExecution, 
                c.coreParameter.startParams.incrementalExecution = null !== (f = d.incremental) && void 0 !== f ? f : c.coreParameter.startParams.incrementalExecution, 
                c.coreParameter.startParams.optimizationStrategy = d.optimizationStrategy && Object.values(c.OptimizationStrategy).includes(d.optimizationStrategy) ? d.optimizationStrategy : c.coreParameter.startParams.optimizationStrategy, 
                a.hvigorTrace.traceBaseConfig(c.coreParameter.startParams.incrementalExecution, c.coreParameter.startParams.daemon, c.coreParameter.startParams.parallelExecution, c.coreParameter.startParams.hvigorfileTypeCheck);
            }(t), p(), function() {
                var e, t;
                const r = null !== (t = null === (e = l.HvigorConfigReader.getHvigorConfig()) || void 0 === e ? void 0 : e.properties) && void 0 !== t ? t : {}, n = Object.fromEntries(Object.entries(r).filter(([e, t]) => void 0 !== t));
                a.hvigorTrace.traceConfigProperties(n);
            }();
        }, e.refreshDaemonProcessEnv = function(e) {
            [ {
                name: "ENABLE_MODULE_SKIP",
                type: "boolean"
            }, {
                name: "ENABLE_CPP_FUNCTION_LEVEL_INCREMENTAL",
                type: "boolean"
            }, {
                name: "DEVECO_SDK_HOME",
                type: "string"
            }, {
                name: "OHOS_BASE_SDK_HOME",
                type: "string"
            } ].forEach(({name: t, type: r}) => {
                var n;
                const o = null === (n = e.env) || void 0 === n ? void 0 : n[t];
                void 0 !== o && (process.env[t] = "boolean" === r ? "true" === o ? "true" : "false" : o);
            });
        }, e.resetStartData = function() {
            e.startEnvironment = {
                ...e.defaultStartEnvironment
            }, c.coreParameter.clean();
        };
        let g = "", m = 0;
    }(oH)), oH;
}

var hH = {}, vH = {};

Object.defineProperty(vH, "__esModule", {
    value: !0
}), vH.MapCacheService = void 0;

vH.MapCacheService = class {
    constructor() {
        this.cacheEntryMap = new Map;
    }
    initialize() {}
    close() {
        this.cacheEntryMap.clear();
    }
    get(e) {
        return this.cacheEntryMap.get(e);
    }
    remove(e) {
        this.cacheEntryMap.delete(e);
    }
    size() {
        return this.cacheEntryMap.size;
    }
}, Object.defineProperty(hH, "__esModule", {
    value: !0
}), hH.MetricCacheService = void 0;

const gH = vH;

class mH extends gH.MapCacheService {
    constructor() {
        super();
    }
    add(e) {
        this.cacheEntryMap.set(e.getId(), e);
    }
    getEvents() {
        const e = [];
        return this.cacheEntryMap.forEach(t => {
            e.push(t);
        }), e;
    }
    static getInstance() {
        return mH.instance || (mH.instance = new mH), mH.instance;
    }
}

hH.MetricCacheService = mH;

var yH, _H, EH, bH = {};

function wH() {
    return yH || (yH = 1, function(e) {
        Object.defineProperty(e, "__esModule", {
            value: !0
        }), e.DurationEvent = e.DurationEventState = void 0;
        const t = WH(), r = NH(), n = DH(), o = Vz, i = SH(), a = Wz;
        var s;
        !function(e) {
            e.CREATED = "created", e.BEGINNING = "beginning", e.RUNNING = "running", e.FAILED = "failed", 
            e.SUCCESS = "success", e.WARN = "warn";
        }(s = e.DurationEventState || (e.DurationEventState = {}));
        class u {
            constructor(e, t) {
                this.children = [], this.state = s.CREATED, this.targetName = "", this.moduleName = "";
                const r = e.indexOf(":");
                if (r > 0) {
                    this.moduleName = e.substring(0, r);
                    const t = e.indexOf("@");
                    t > 0 && (this.targetName = e.substring(r + 1, t));
                }
                this.category = t, this.taskRunReasons = [];
            }
        }
        class l extends o.BaseEvent {
            constructor(e, r, n, i, a, s) {
                super(new o.EventHead(e, r, n, o.MetricEventType.DURATION), new o.EventBody(i, s)), 
                this.log = t.HvigorLogger.getLogger("DurationEvent"), this.additional = new u(r, a);
            }
            start(e = s.RUNNING, t) {
                return this.setState(e), super.setStartTime(t), this;
            }
            stop(e = s.SUCCESS, t) {
                if (this.additional.state === s.FAILED || this.additional.state === s.SUCCESS || this.additional.state === s.WARN) {
                    return this;
                }
                this.body.endTime = null != t ? t : Number(process.hrtime.bigint());
                const r = n.MetricService.getInstance();
                this.setState(e);
                for (const t of this.additional.children) {
                    const n = r.getEventById(t);
                    n ? n instanceof l ? n.stop(e) : this.log.warn(`Child:'${t}' is not of type DurationEvent.`) : this.log.warn(`Can not getEventById:'${t}' from MetricCacheService.`);
                }
                return this;
            }
            setState(e) {
                this.additional.state = e;
            }
            createSubEvent(e, t) {
                const n = r.MetricFactory.createDurationEvent(e, t, "");
                return n.setParent(this.getId()), this.addChild(n.getId()), n;
            }
            addChild(e) {
                this.additional.children.push(e);
            }
            setParent(e) {
                this.additional.parent = e;
            }
            getParent() {
                return this.additional.parent;
            }
            getChildren() {
                return this.additional.children;
            }
            setLog(e, t = a.MetricLogType.INFO, n, o) {
                const i = r.MetricFactory.createLogEvent(null != e ? e : this.head.name, t, this.getTid(), n);
                i.setDurationId(this.getId()), this.additional.logId = i.getId(), i.setStartTime(this.body.startTime), 
                i.setEndTime(this.body.endTime), o && i.setTotalTime(o), this.setParentLog(i), this.setChildrenLog(i);
            }
            setParentLog(e) {
                const t = n.MetricService.getInstance().getEventById(this.additional.parent);
                if (t instanceof l) {
                    const r = n.MetricService.getInstance().getEventById(t.additional.logId);
                    r instanceof a.LogEvent && (r.addChild(e.getId()), e.setParent(r.getId()));
                }
            }
            setChildrenLog(e) {
                this.additional.children.forEach(t => {
                    const r = n.MetricService.getInstance().getEventById(t);
                    if (r instanceof l || r instanceof i.ContinualEvent) {
                        e.addChild(r.additional.logId);
                        const t = n.MetricService.getInstance().getEventById(r.additional.logId);
                        t instanceof a.LogEvent && r.setParentLog(t);
                    }
                });
            }
            setDetail(e) {
                const t = r.MetricFactory.createLogEvent(e, a.MetricLogType.DETAIL, this.getTid());
                t.setDurationId(this.getId()), this.additional.detailId = t.getId();
            }
            setCategory(e) {
                this.additional.category = e;
            }
            addTaskRunReason(e) {
                this.additional.taskRunReasons.push(e);
            }
        }
        e.DurationEvent = l;
    }(bH)), bH;
}

function DH() {
    if (_H) {
        return Jz;
    }
    _H = 1, Object.defineProperty(Jz, "__esModule", {
        value: !0
    }), Jz.MetricService = void 0;
    const e = Xz, o = function() {
        if (lH) {
            return Zz;
        }
        lH = 1;
        var e = g && g.__importDefault || function(e) {
            return e && e.__esModule ? e : {
                default: e
            };
        };
        Object.defineProperty(Zz, "__esModule", {
            value: !0
        }), Zz.ReportServiceImpl = void 0;
        const o = e(t), i = e(r), a = e(n), s = e(Bs), u = Yz, l = SU, c = LU;
        class f {
            constructor() {
                this.reportListeners = [];
            }
            report() {
                const e = this.getReport(), t = l.PathUtil.getReportDirPath();
                o.default.existsSync(t) || o.default.mkdirSync(t, {
                    recursive: !0
                }), this.deleteUnusableFiles(t), this.storage(e, t);
            }
            getReport() {
                const e = {
                    version: "2.0",
                    ppid: a.default.ppid
                };
                for (const t of this.reportListeners) {
                    const r = t.queryReport();
                    e[r.getName()] = r.getValue();
                }
                return e;
            }
            storage(e, t) {
                const r = o.default.readdirSync(t).filter(e => e.startsWith("report-") && e.endsWith("json")).sort((e, r) => {
                    const n = i.default.resolve(t, e), a = i.default.resolve(t, r), s = o.default.statSync(n);
                    return o.default.statSync(a).birthtimeMs - s.birthtimeMs;
                });
                for (let e = 0; e < r.length; e++) {
                    if (e >= 9) {
                        const n = i.default.resolve(t, r[e]);
                        o.default.existsSync(n) && o.default.unlinkSync(n);
                    }
                }
                const n = pH();
                if (void 0 === n.globalData.buildId) {
                    return;
                }
                const a = n.globalData.buildId;
                f.buildId = a;
                const s = i.default.resolve(t, `report-${a}.json`);
                u.LocalFileWriter.getInstance().write(s, e), d() && this.generateHtmlResource(t, `report-${a}`, e);
            }
            deleteUnusableFiles(e) {
                o.default.readdirSync(e).forEach(t => {
                    if (!f.REPORT_REG.test(t) && (!f.HTML_REG.test(t) || d()) && t !== f.HTML_RESOURCE_NAME && t !== f.UPLOAD_NAME) {
                        const r = i.default.resolve(e, t);
                        o.default.existsSync(r) && o.default.unlinkSync(r);
                    }
                });
            }
            addListener(e) {
                this.reportListeners.push(e);
            }
            removeListener(e) {
                const t = this.reportListeners.indexOf(e);
                -1 !== t && this.reportListeners.splice(t, 1);
            }
            generateHtmlResource(e, t, r) {
                const n = i.default.resolve(e, "htmlResource"), a = i.default.resolve(__filename, "../../../../../res/staticHtmlResource/htmlResource");
                if (o.default.existsSync(n)) {
                    const e = o.default.readdirSync(a), t = o.default.readdirSync(n);
                    e.every(e => !!t.includes(e) && o.default.statSync(i.default.resolve(a, e)).size === o.default.statSync(i.default.resolve(n, e)).size) || s.default.copySync(a, n);
                } else {
                    s.default.copySync(a, n);
                }
                const u = i.default.resolve(__filename, "../../../../../res/staticHtmlResource/index.html"), l = o.default.readFileSync(u, "utf8"), c = `<script>window.__HVIGOR_REPORT__ = ${JSON.stringify(JSON.stringify(r))};<\/script>`, f = l.indexOf("</body>"), d = l.slice(0, f) + c + l.slice(f), p = i.default.resolve(e, `${t}.html`);
                o.default.writeFileSync(p, d);
            }
            static getInstance() {
                return f.instance || (f.instance = new f), f.instance;
            }
            startProcessMonitor() {
                this.monitorTimeId && clearInterval(this.monitorTimeId), f.data = [], this.monitorTimeId = setInterval(() => {
                    const e = a.default.memoryUsage(), t = {
                        time: Date.now(),
                        rss: this.convertToMb(e.rss),
                        heapTotal: this.convertToMb(e.heapTotal),
                        heapUsed: this.convertToMb(e.heapUsed),
                        external: this.convertToMb(e.external),
                        arrayBuffers: this.convertToMb(e.arrayBuffers)
                    };
                    f.data.push(t);
                }, f.REPORT_INTERVAL_MS);
            }
            stopProcessMonitor() {
                if (this.monitorTimeId) {
                    clearInterval(this.monitorTimeId), this.monitorTimeId = void 0;
                    const e = {
                        version: f.VERSION,
                        pid: a.default.pid,
                        data: f.data
                    };
                    if (void 0 === f.buildId) {
                        return;
                    }
                    const t = l.PathUtil.getReportDirPath(), r = i.default.join(t, `report-monitor-${f.buildId}.json`);
                    o.default.writeFileSync(r, JSON.stringify(e, null, 2), "utf-8");
                }
            }
            convertToMb(e) {
                return e / f.MB_CONVERTER / f.MB_CONVERTER;
            }
        }
        function d() {
            return "boolean" == typeof c.coreParameter.properties["hvigor.analyzeHtml"] && c.coreParameter.properties["hvigor.analyzeHtml"];
        }
        return Zz.ReportServiceImpl = f, f.MAX_REPEAT_TIMES = 10, f.REPORT_REG = /^report(-monitor)?-[0-9]+\.json$/, 
        f.HTML_REG = /^report-?[0-9]*.html$/, f.HTML_RESOURCE_NAME = "htmlResource", f.UPLOAD_NAME = "upload", 
        f.VERSION = "1.0", f.MB_CONVERTER = 1024, f.REPORT_INTERVAL_MS = 1e3, f.data = [], 
        Zz;
    }(), i = hH, a = Vz, s = wH(), u = Wz;
    class l {
        constructor() {
            this.metricCacheService = i.MetricCacheService.getInstance();
        }
        submit(e) {
            this.metricCacheService.add(e);
        }
        getEventById(e) {
            if (e) {
                return this.metricCacheService.get(e);
            }
        }
        queryReport() {
            let t = this.filterDurationEvent(this.metricCacheService.getEvents());
            return t = this.filterLogEvent(t), new e.Report("events", t);
        }
        filterDurationEvent(e) {
            return e.filter(e => {
                if (e.getType() === a.MetricEventType.DURATION) {
                    if (e.additional.state === s.DurationEventState.CREATED) {
                        return !1;
                    }
                }
                return !0;
            });
        }
        filterLogEvent(e) {
            return e.filter(e => {
                if (e.getType() === a.MetricEventType.LOG) {
                    const t = e, r = this.getEventById(t.additional.durationId);
                    if (r && r.additional.state === s.DurationEventState.CREATED) {
                        return !1;
                    }
                    if (t.additional.logType === u.MetricLogType.DETAIL && !r) {
                        return !1;
                    }
                }
                return !0;
            });
        }
        clear() {
            this.metricCacheService.close();
        }
        static getInstance() {
            return l.instance || (l.instance = new l, o.ReportServiceImpl.getInstance().addListener(l.instance)), 
            l.instance;
        }
    }
    return Jz.MetricService = l, Jz;
}

function SH() {
    if (EH) {
        return qz;
    }
    EH = 1, Object.defineProperty(qz, "__esModule", {
        value: !0
    }), qz.ContinualEvent = qz.ContinualEventAdditional = void 0;
    const e = NH(), t = DH(), r = Vz, n = wH(), o = Wz;
    class i {
        constructor(e, t) {
            this.totalTime = null != e ? e : 0, this.frequency = null != t ? t : 0, this.children = [];
        }
    }
    qz.ContinualEventAdditional = i;
    class a extends r.BaseEvent {
        constructor(e, t, n, o, a, s, u) {
            super(new r.EventHead(e, t, n, r.MetricEventType.CONTINUAL), new r.EventBody(o, a)), 
            this.additional = new i(s, u);
        }
        setParent(e) {
            this.additional.parent = e;
        }
        getParent() {
            return this.additional.parent;
        }
        addChild(e) {
            this.additional.children.push(e);
        }
        getChildren() {
            return this.additional.children;
        }
        createSubEvent(t, r) {
            const n = e.MetricFactory.createContinualEvent(t, r);
            return n.setParent(this.getId()), this.addChild(n.getId()), n;
        }
        setLog(t, r, n) {
            const o = e.MetricFactory.createLogEvent(t, r, this.getTid(), n);
            o.setContinualId(this.getId()), this.additional.logId = o.getId(), o.setStartTime(this.body.startTime), 
            o.setEndTime(this.body.endTime), this.setParentLog(o), this.setChildrenLog(o);
        }
        setParentLog(e) {
            const r = t.MetricService.getInstance().getEventById(this.additional.parent);
            if (r instanceof a || r instanceof n.DurationEvent) {
                const n = t.MetricService.getInstance().getEventById(r.additional.logId);
                n instanceof o.LogEvent && (n.addChild(e.getId()), e.setParent(n.getId()));
            }
        }
        setDetail(t) {
            const r = e.MetricFactory.createLogEvent(t, o.MetricLogType.DETAIL, this.getTid());
            r.setContinualId(this.getId()), this.additional.detailId = r.getId();
        }
        setChildrenLog(e) {
            this.additional.children.forEach(r => {
                const n = t.MetricService.getInstance().getEventById(r);
                if (n instanceof a) {
                    e.addChild(n.additional.logId);
                    const r = t.MetricService.getInstance().getEventById(n.additional.logId);
                    r instanceof o.LogEvent && n.setParentLog(r);
                }
            });
        }
    }
    return qz.ContinualEvent = a, qz;
}

var AH = {};

Object.defineProperty(AH, "__esModule", {
    value: !0
}), AH.CounterEvent = AH.CounterEventAdditional = void 0;

const OH = Vz;

class CH {
    constructor(e, t) {
        this.success = null != e ? e : 0, this.failed = null != t ? t : 0;
    }
}

AH.CounterEventAdditional = CH;

class xH extends OH.BaseEvent {
    constructor(e, t, r, n, o, i, a) {
        super(new OH.EventHead(e, t, r, OH.MetricEventType.COUNTER), new OH.EventBody(n, o)), 
        this.additional = new CH(i, a), this.body.startTime = Number(process.hrtime.bigint());
    }
}

AH.CounterEvent = xH;

var FH = {};

Object.defineProperty(FH, "__esModule", {
    value: !0
}), FH.GaugeEvent = FH.GaugeEventAdditional = void 0;

const MH = Vz;

class PH {
    constructor(e) {
        this.utilization = e;
    }
}

FH.GaugeEventAdditional = PH;

class IH extends MH.BaseEvent {
    constructor(e, t, r, n, o, i) {
        super(new MH.EventHead(e, t, r, MH.MetricEventType.GAUGE), new MH.EventBody(n, o)), 
        this.additional = new PH(i), this.body.startTime = Number(process.hrtime.bigint());
    }
}

FH.GaugeEvent = IH;

var kH = {};

!function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.InstantEvent = e.InstantEventAdditional = e.InstantEventScope = void 0;
    const t = Vz;
    var r;
    (r = e.InstantEventScope || (e.InstantEventScope = {})).THREAD = "thread", r.PROCESS = "process", 
    r.GLOBAL = "global";
    class n {}
    e.InstantEventAdditional = n;
    class o extends t.BaseEvent {
        constructor(e, r, o, i, a) {
            super(new t.EventHead(e, r, o, t.MetricEventType.INSTANT), new t.EventBody(i, a)), 
            this.additional = new n, this.body.startTime = Number(process.hrtime.bigint());
        }
        setScope(e) {
            this.additional.scope = e;
        }
    }
    e.InstantEvent = o;
}(kH);

var RH = {};

!function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.MarkEvent = e.MarkEventAdditional = e.MarkEventTime = e.MarkEventState = e.MarkEventCategory = e.MarkEventType = void 0;
    const t = Vz;
    var r, n, o;
    (r = e.MarkEventType || (e.MarkEventType = {})).HISTORY = "history", r.OTHER = "other", 
    (n = e.MarkEventCategory || (e.MarkEventCategory = {})).BUILD = "build", n.CLEAN = "clean", 
    function(e) {
        e.SUCCESS = "success", e.FAILED = "failed", e.RUNNING = "running";
    }(o = e.MarkEventState || (e.MarkEventState = {}));
    class i {
        constructor(e) {
            this.year = e.getFullYear(), this.month = e.getMonth() + 1, this.day = e.getDate(), 
            this.hour = e.getHours(), this.minute = e.getMinutes(), this.second = e.getSeconds();
        }
    }
    e.MarkEventTime = i;
    class a {
        constructor() {
            this.time = new i(new Date);
        }
    }
    e.MarkEventAdditional = a;
    class s extends t.BaseEvent {
        constructor(e, r, n, o, i) {
            super(new t.EventHead(e, r, n, t.MetricEventType.MARK), new t.EventBody(o, i)), 
            this.additional = new a;
        }
        start(e = o.RUNNING, t) {
            this.setState(e), super.setStartTime(t);
        }
        stop(e = o.SUCCESS, t) {
            this.additional.state !== o.FAILED && this.additional.state !== o.SUCCESS && (this.body.endTime = null != t ? t : Number(process.hrtime.bigint()), 
            this.setState(e));
        }
        setMarkType(e) {
            this.additional.markType = e;
        }
        setCategory(e) {
            this.additional.category = e;
        }
        setState(e) {
            this.additional.state = e;
        }
        setHvigorVersion(e) {
            this.additional.hvigorVersion = e;
        }
        setCompleteCommand(e) {
            this.additional.completeCommand = e;
        }
        setNodeVersion(e) {
            this.additional.nodeVersion = e;
        }
    }
    e.MarkEvent = s;
}(RH);

var TH = {};

!function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.MetadataEvent = e.MetadataEventState = void 0;
    const t = Vz;
    var r;
    (r = e.MetadataEventState || (e.MetadataEventState = {})).NEW = "new", r.IDLE = "idle", 
    r.BUSY = "busy", r.CLOSE = "close", r.BROKEN = "broken";
    class n {
        constructor(e) {
            this.state = e;
        }
    }
    class o extends t.BaseEvent {
        constructor(e, r, o, i, a, s) {
            super(new t.EventHead(e, r, o, t.MetricEventType.METADATA), new t.EventBody(i, a)), 
            this.additional = new n(s), this.body.startTime = Number(process.hrtime.bigint());
        }
        setCategory(e) {
            this.additional.category = e;
        }
        setSortIndex(e) {
            this.additional.sortIndex = e;
        }
        setLabel(e) {
            this.additional.label = e;
        }
        setContent(e) {
            this.additional.content = e;
        }
    }
    e.MetadataEvent = o;
}(TH);

var jH, LH = {};

function NH() {
    return jH || (jH = 1, function(e) {
        var t = g && g.__importDefault || function(e) {
            return e && e.__esModule ? e : {
                default: e
            };
        };
        Object.defineProperty(e, "__esModule", {
            value: !0
        }), e.MetricFactory = e.MAIN_THREAD = void 0;
        const r = t(o), n = SH(), i = AH, a = wH(), s = FH, u = kH, l = Wz, c = RH, f = TH, d = LH, p = DH();
        e.MAIN_THREAD = "Main Thread";
        class h {
            static getUuid() {
                return r.default.randomUUID();
            }
            static createDurationEvent(t, r, n, o) {
                const i = new a.DurationEvent(h.getUuid(), t, r, process.pid, n, null != o ? o : e.MAIN_THREAD);
                return p.MetricService.getInstance().submit(i), i;
            }
            static createInstantEvent(t, r, n) {
                const o = new u.InstantEvent(h.getUuid(), t, r, process.pid, null != n ? n : e.MAIN_THREAD);
                return p.MetricService.getInstance().submit(o), o;
            }
            static createCounterEvent(t, r, n, o, a) {
                const s = new i.CounterEvent(h.getUuid(), t, r, process.pid, null != a ? a : e.MAIN_THREAD, n, o);
                return p.MetricService.getInstance().submit(s), s;
            }
            static createGaugeEvent(t, r, n, o) {
                const i = new s.GaugeEvent(h.getUuid(), t, n, process.pid, null != o ? o : e.MAIN_THREAD, r);
                return p.MetricService.getInstance().submit(i), i;
            }
            static createObjectEvent(t, r, n, o, i, a) {
                const s = new d.ObjectEvent(h.getUuid(), t, o, process.pid, null != a ? a : e.MAIN_THREAD, r, n, i);
                return p.MetricService.getInstance().submit(s), s;
            }
            static createMetadataEvent(t, r, n, o) {
                const i = new f.MetadataEvent(h.getUuid(), t, n, process.pid, null != o ? o : e.MAIN_THREAD, r);
                return p.MetricService.getInstance().submit(i), i;
            }
            static createMarkEvent(t, r, n) {
                const o = new c.MarkEvent(h.getUuid(), t, r, process.pid, null != n ? n : e.MAIN_THREAD);
                return p.MetricService.getInstance().submit(o), o;
            }
            static createLogEvent(t, r, n, o) {
                const i = new l.LogEvent(h.getUuid(), t, null != o ? o : "", process.pid, null != n ? n : e.MAIN_THREAD, r);
                return p.MetricService.getInstance().submit(i), i;
            }
            static createContinualEvent(t, r, o, i, a) {
                const s = new n.ContinualEvent(h.getUuid(), t, r, process.pid, null != a ? a : e.MAIN_THREAD, o, i);
                return p.MetricService.getInstance().submit(s), s;
            }
        }
        e.MetricFactory = h;
    }(Kz)), Kz;
}

!function(e) {
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.ObjectEvent = e.ObjectEventAdditional = e.ObjectEventState = void 0;
    const t = Vz;
    var r;
    (r = e.ObjectEventState || (e.ObjectEventState = {})).NEW = "new", r.SNAPSHOT = "snapshot", 
    r.DESTROY = "destroy";
    class n {
        constructor(e, t, r) {
            this.objectId = e, this.state = t, this.snapshot = r;
        }
    }
    e.ObjectEventAdditional = n;
    class o extends t.BaseEvent {
        constructor(e, r, o, i, a, s, u, l) {
            super(new t.EventHead(e, r, o, t.MetricEventType.OBJECT), new t.EventBody(i, a)), 
            this.additional = new n(s, u, l), this.body.startTime = Number(process.hrtime.bigint());
        }
    }
    e.ObjectEvent = o;
}(LH);

var BH, UH = {}, zH = g && g.__decorate || function(e, t, r, n) {
    var o, i = arguments.length, a = i < 3 ? t : null === n ? n = Object.getOwnPropertyDescriptor(t, r) : n;
    if ("object" == typeof Reflect && "function" == typeof Reflect.decorate) {
        a = Reflect.decorate(e, t, r, n);
    } else {
        for (var s = e.length - 1; s >= 0; s--) {
            (o = e[s]) && (a = (i < 3 ? o(a) : i > 3 ? o(t, r, a) : o(t, r)) || a);
        }
    }
    return i > 3 && a && Object.defineProperty(t, r, a), a;
};

function HH(e, t, r) {
    const n = r.value;
    return r.value = function(...e) {
        const t = $H(e);
        return n.apply(this, t);
    }, r;
}

function $H(e) {
    if ("object" != typeof e) {
        return e;
    }
    if (Array.isArray(e)) {
        return e.map((t, r) => "object" == typeof t ? $H(t) : e[r]);
    }
    if ("object" == typeof e) {
        const t = {};
        return Object.keys(e).forEach(r => {
            if ("bundleName" === r && "string" == typeof e[r]) {
                const n = e[r];
                t[r] = n ? `${n[0]}***${n[n.length - 1]}` : "*****";
            } else {
                "object" == typeof e[r] ? t[r] = $H(e[r]) : t[r] = e[r];
            }
        }), t;
    }
    return e;
}

Object.defineProperty(UH, "__esModule", {
    value: !0
}), UH.FileLogger = UH.replaceBundleName = void 0, UH.replaceBundleName = function e(t, r, n) {
    if (!(null == r ? void 0 : r.length)) {
        return t;
    }
    if (n || (n = new RegExp(r, "ig")), "string" == typeof t && n.test(t)) {
        return t.replace(n, e => `${e[0]}***${e[e.length - 1]}`);
    }
    if (Array.isArray(t)) {
        return t.map(t => e(t, r, n));
    }
    if ("object" == typeof t) {
        return Object.keys(t).reduce((o, i) => ({
            ...o,
            [i]: e(t[i], r, n)
        }), {});
    }
    return t;
};

class GH {
    constructor(e) {
        this.fileLogger = e;
    }
    debug(e, ...t) {
        return this.fileLogger.debug(e, ...t), [ e, ...t ];
    }
    log(e, ...t) {
        this.fileLogger.log(e, ...t);
    }
    warn(e, ...t) {
        this.fileLogger.warn(e, ...t);
    }
    info(e, ...t) {
        this.fileLogger.info(e, ...t);
    }
    error(e, ...t) {
        this.fileLogger.error(e, ...t);
    }
}

function WH() {
    if (BH) {
        return wc;
    }
    BH = 1;
    var e = g && g.__createBinding || (Object.create ? function(e, t, r, n) {
        void 0 === n && (n = r);
        var o = Object.getOwnPropertyDescriptor(t, r);
        o && !("get" in o ? !t.__esModule : o.writable || o.configurable) || (o = {
            enumerable: !0,
            get: function() {
                return t[r];
            }
        }), Object.defineProperty(e, n, o);
    } : function(e, t, r, n) {
        void 0 === n && (n = r), e[n] = t[r];
    }), t = g && g.__setModuleDefault || (Object.create ? function(e, t) {
        Object.defineProperty(e, "default", {
            enumerable: !0,
            value: t
        });
    } : function(e, t) {
        e.default = t;
    }), r = g && g.__importStar || function(r) {
        if (r && r.__esModule) {
            return r;
        }
        var n = {};
        if (null != r) {
            for (var o in r) {
                "default" !== o && Object.prototype.hasOwnProperty.call(r, o) && e(n, r, o);
            }
        }
        return t(n, r), n;
    };
    Object.defineProperty(wc, "__esModule", {
        value: !0
    }), wc.configure = wc.evaluateLogLevel = wc.HvigorLogger = void 0;
    const n = r(l), o = Dc, i = r(Gm), a = Ny, s = Wz, u = NH(), c = UH, f = iH;
    class d {
        constructor(e) {
            i.configure((0, f.updateConfiguration)()), this._logger = i.getLogger(e), this._logger.level = (0, 
            f.getLevel)(), this._filelogger = i.getLogger("debug-file"), this.anonymizeFileLogger = new c.FileLogger(i.getLogger("debug-file"));
        }
        static getInstance(e, t) {
            const r = `${e.name}:${t}`;
            return this.instanceMap.has(r) || this.instanceMap.set(r, new e(t)), this.instanceMap.get(r);
        }
        static getLogger(e) {
            return this.getInstance(d, e);
        }
        static getLoggerWithDurationId(e, t) {
            const r = {
                ...this.getInstance(d, e)
            }, n = Object.setPrototypeOf(r, d.prototype);
            return n.durationId = t, n;
        }
        static clean() {
            d.instanceMap.clear();
        }
        log(e, ...t) {
            this.createLogEventByDurationId(e, s.MetricLogType.INFO, ...t), this._logger.log(e, ...t), 
            this._filelogger.log(e, ...t);
        }
        debug(e, ...t) {
            this.createLogEventByDurationId(e, s.MetricLogType.DEBUG, ...t), this._logger.debug(e, ...t), 
            this._filelogger.debug(e, ...t);
        }
        info(e, ...t) {
            this.createLogEventByDurationId(e, s.MetricLogType.INFO, ...t), this._logger.info(e, ...t), 
            this._filelogger.info(e, ...t);
        }
        warn(e, ...t) {
            void 0 !== e && "" !== e && (this.createLogEventByDurationId(e, s.MetricLogType.WARN, ...t), 
            this._logger.warn(e, ...t), this._filelogger.warn(e, ...t));
        }
        error(e, ...t) {
            this.createLogEventByDurationId(e, s.MetricLogType.ERROR, ...t), this._logger.error(e, ...t), 
            this._filelogger.error(e, ...t);
        }
        anonymizeDebug(e, ...t) {
            this._logger.debug(e, ...t);
            const [r, ...n] = this.anonymizeFileLogger.debug(e, ...t);
            this.createLogEventByDurationId(r, s.MetricLogType.DEBUG, ...n);
        }
        _printTaskExecuteInfo(e, t) {
            this._logger.info(`Finished :${e}... after ${t}`), this._filelogger.info(`Finished :${e}... after ${t}`);
        }
        _printFailedTaskInfo(e) {
            this._logger.error(`Failed :${e}... `), this._filelogger.error(`Failed :${e}... `);
        }
        _printDisabledTaskInfo(e) {
            this._logger.info(`Disabled :${e}... `), this._filelogger.info(`Disabled :${e}... `);
        }
        _printUpToDateTaskInfo(e) {
            this._logger.info(`UP-TO-DATE :${e}...  `), this._filelogger.info(`UP-TO-DATE :${e}...  `);
        }
        _printStackErrorToFile(e, ...t) {
            this._filelogger.error(e, ...t);
        }
        errorMessageExit(e, ...t) {
            throw new Error(n.format(e, ...t));
        }
        errorExit(e, t, ...r) {
            if (t && (u.MetricFactory.createLogEvent(this.getMessage(t, ...r), s.MetricLogType.ERROR), 
            this._logger.error(t, r), this._filelogger.error(t, r)), this._logger.error(e.stack), 
            this._filelogger.error(e.stack), e.stack) {
                throw u.MetricFactory.createLogEvent(e.stack, s.MetricLogType.ERROR), e;
            }
        }
        getLevel() {
            return this._logger.level;
        }
        setLevel(e) {
            this._logger.level = e;
        }
        createLogEventByDurationId(e, t, ...r) {
            if ("string" == typeof e) {
                const n = u.MetricFactory.createLogEvent(this.getMessage(e, ...r), t);
                this.durationId && n.setDurationId(this.durationId);
            }
            return e;
        }
        getMessage(e, ...t) {
            return t.length > 0 ? n.format(e, ...t) : e;
        }
        getAdaptor(e) {
            return new o.HvigorErrorAdaptor(e);
        }
        combinePhase(e) {
            return a.hvigorTrace.traceErrorMessage(e), e.solutions ? o.ErrorUtil.combinePhase({
                code: e.code,
                cause: e.message,
                position: "",
                solutions: e.solutions,
                moreInfo: e.moreInfo
            }) : e.message;
        }
        formatErrorAdaptor(e, t, r) {
            let n = this.getAdaptor(e);
            return t && (n = n.formatMessage(...t)), r && r.forEach((e, t) => {
                n = n.formatSolutions(t, ...e);
            }), n;
        }
        printErrorWithAdaptorErrorMessage(e, t = "") {
            const r = this.combinePhase(e[0]);
            this._logger.error(r + t);
            for (let t = 1; t < e.length; ++t) {
                this.combinePhase(e[t]);
            }
        }
        printError(e, t, r) {
            const n = this.formatErrorAdaptor(e, t, r);
            this.printErrorWithAdaptorErrorMessage(n.getErrorMessage());
        }
        printErrorExit(e, t, r, n) {
            const i = this.formatErrorAdaptor(e, t, r), a = this.combinePhase(o.ErrorUtil.getFirstErrorAdaptorMessage(i.getErrorMessage()));
            throw new o.AdaptorError(a, n);
        }
        printErrorExitWithoutStack(e, t, r) {
            this.printError(e, t, r), process.exit(-1);
        }
    }
    return wc.HvigorLogger = d, d.instanceMap = new Map, wc.evaluateLogLevel = function(e, t) {
        (0, f.setCategoriesLevel)(e, t), i.shutdown(), i.configure((0, f.updateConfiguration)());
    }, wc.configure = function(e) {
        const t = (0, f.getConfiguration)(), r = {
            appenders: {
                ...t.appenders,
                ...e.appenders
            },
            categories: {
                ...t.categories,
                ...e.categories
            }
        };
        (0, f.setConfiguration)(r), i.shutdown(), i.configure(r);
    }, wc;
}

zH([ HH ], GH.prototype, "debug", null), zH([ HH ], GH.prototype, "log", null), 
zH([ HH ], GH.prototype, "warn", null), zH([ HH ], GH.prototype, "info", null), 
zH([ HH ], GH.prototype, "error", null), UH.FileLogger = GH;

var VH = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(bc, "__esModule", {
    value: !0
}), bc.calcChildExecArgv = void 0;

const KH = VH(n), qH = WH(), JH = dH(), XH = CU, ZH = qH.HvigorLogger.getLogger(JH.HvigorConfigReader.name);

bc.calcChildExecArgv = function(e = !1) {
    var t, r;
    const n = [ ...KH.default.execArgv ], o = `--max-old-space-size=${JH.HvigorConfigReader.getMaxOldSpaceSize(e)}`, i = n.findIndex(e => e.startsWith("--max-old-space-size="));
    -1 !== i && n[i] ? n[i] = o : n.push(o);
    const a = `--max-semi-space-size=${JH.HvigorConfigReader.getMaxSemiSpaceSize(e)}`, s = n.findIndex(e => e.startsWith("--max-semi-space-size="));
    -1 !== s && n[s] ? n[s] = a : n.push(a);
    const u = null === (r = null === (t = JH.HvigorConfigReader.getHvigorConfig()) || void 0 === t ? void 0 : t.nodeOptions) || void 0 === r ? void 0 : r.exposeGC, l = n.indexOf("--expose-gc");
    !1 === u || -1 !== l && n[l] ? !1 !== u || -1 === l && !n[l] || n.splice(l, 1) : n.push("--expose-gc");
    const c = JH.HvigorConfigReader.getStacktrace(e);
    return n.some(e => e === XH.ENABLE_SOURCE_MAPS) ? !c && ZH.warn(`"${XH.ENABLE_SOURCE_MAPS}" is enabled but has no effect because "stacktrace" is disabled. Please enable "stacktrace" first.`) : c && n.push(XH.ENABLE_SOURCE_MAPS), 
    n;
};

var YH = g && g.__createBinding || (Object.create ? function(e, t, r, n) {
    void 0 === n && (n = r);
    var o = Object.getOwnPropertyDescriptor(t, r);
    o && !("get" in o ? !t.__esModule : o.writable || o.configurable) || (o = {
        enumerable: !0,
        get: function() {
            return t[r];
        }
    }), Object.defineProperty(e, n, o);
} : function(e, t, r, n) {
    void 0 === n && (n = r), e[n] = t[r];
}), QH = g && g.__setModuleDefault || (Object.create ? function(e, t) {
    Object.defineProperty(e, "default", {
        enumerable: !0,
        value: t
    });
} : function(e, t) {
    e.default = t;
}), e$ = g && g.__importStar || function(e) {
    if (e && e.__esModule) {
        return e;
    }
    var t = {};
    if (null != e) {
        for (var r in e) {
            "default" !== r && Object.prototype.hasOwnProperty.call(e, r) && YH(t, e, r);
        }
    }
    return QH(t, e), t;
};

Object.defineProperty(hc, "__esModule", {
    value: !0
}), hc.executeBuild = void 0;

const t$ = i, r$ = e$(t), n$ = e$(r), o$ = vc, i$ = bc;

hc.executeBuild = function(e) {
    var t, r;
    const n = n$.resolve(e, "node_modules", "@ohos", "hvigor", "bin", "hvigor.js");
    try {
        const e = r$.realpathSync(n), o = process.argv.slice(2), i = (0, t$.fork)(e, o, {
            env: process.env,
            execArgv: (0, i$.calcChildExecArgv)()
        });
        null === (t = i.stdout) || void 0 === t || t.on("data", e => {
            (0, o$.logInfo)(`${e.toString().trim()}`);
        }), null === (r = i.stderr) || void 0 === r || r.on("data", e => {
            (0, o$.logError)(`${e.toString().trim()}`);
        }), i.on("exit", (e, t) => {
            process.exit(null != e ? e : -1);
        });
    } catch (t) {
        (0, o$.logFormatedErrorAndExit)("00308003", `ENOENT: no such file ${n}`, [ `delete ${e} and retry.` ]);
    }
};

var a$ = {}, s$ = {};

Object.defineProperty(s$, "__esModule", {
    value: !0
}), s$.exit = void 0, s$.exit = function(e) {
    "win32" === process.platform && process.stdout.writableLength ? process.stdout.once("drain", function() {
        process.exit(e);
    }) : process.exit(e);
};

var u$ = {}, l$ = "6.21.1", c$ = {
    options: {
        version: {
            name: "version",
            flag: "-v, --version",
            description: "Shows the version of Hvigor."
        },
        usage: {
            name: "hvigor",
            flag: "hvigor",
            description: "[taskNames...] <options...>"
        },
        basicOptions: [ {
            name: "error",
            flag: "-e, --error",
            description: "Sets the log level to error."
        }, {
            name: "warn",
            flag: "-w, --warn",
            description: "Sets the log level to warn."
        }, {
            name: "info",
            flag: "-i, --info",
            description: "Sets the log level to info."
        }, {
            name: "debug",
            flag: "-d, --debug",
            description: "Sets the log level to debug."
        }, {
            name: "config",
            flag: "-c, --config <config>",
            description: "Sets properties in the hvigor-config.json5 file. The settings will overwrite those in the file.",
            back: "array"
        }, {
            name: "prop",
            flag: "-p, --prop <value>",
            description: "Defines extra properties. (default: [])",
            back: "array"
        }, {
            name: "mode",
            flag: "-m, --mode <string>",
            description: "Sets the mode in which the command is executed."
        }, {
            name: "sync",
            flag: "-s, --sync",
            description: "Syncs the information in plugin for other platforms."
        }, {
            name: "nodeHome",
            flag: "--node-home, <string>",
            description: "Sets the Node.js location."
        }, {
            name: "stopDaemon",
            flag: "--stop-daemon",
            description: "Stops the current project's daemon process."
        }, {
            name: "stopDaemonAll",
            flag: "--stop-daemon-all",
            description: "Stops all projects' daemon process."
        }, {
            name: "statusDaemon",
            flag: "--status-daemon",
            description: "Shows the daemon process status of the current project."
        }, {
            name: "verboseAnalyze",
            flag: "--verbose-analyze",
            description: "Enables detailed mode for build analysis."
        }, {
            name: "watch",
            flag: "--watch",
            description: "Enables watch mode."
        }, {
            name: "hotCompile",
            flag: "--hot-compile",
            description: "HotReload watch mode to compile."
        }, {
            name: "hotBuild",
            flag: "--hot-reload-build",
            description: "HotReload build"
        }, {
            name: "maxOldSpaceSize",
            flag: "--max-old-space-size <integer>",
            description: "Sets the maximum memory size of V8's old memory section.",
            back: "number"
        }, {
            name: "maxSemiSpaceSize",
            flag: "--max-semi-space-size <integer>",
            description: "Sets the maximum memory size of V8's new space memory section."
        } ],
        otherOptions: [ {
            name: "xmx",
            flag: "--Xmx <integer>",
            description: "Sets the maximum JVM heap size, in MB.",
            back: "number"
        }, {
            name: "optimizationStrategy",
            flag: "--optimization-strategy <string>",
            description: "Sets the optimization strategy: memory, performance."
        }, {
            name: "enableTypeCheck",
            flag: "--enable-build-script-type-check",
            deprecated: !0,
            recommendedFlags: "type-check",
            description: "['--enable-build-script-type-check' deprecated: use 'type-check' instead] Enables the build script hvigorfile.ts type check. This option is deprecated. Use 'type-check' instead."
        }, {
            name: "stacktrace",
            flag: "stacktrace",
            description: "the printing of stack traces for all exceptions.",
            flagPair: !0
        }, {
            name: "typeCheck",
            flag: "type-check",
            description: "the build script hvigorfile.ts type check.",
            flagPair: !0
        }, {
            name: "parallel",
            flag: "parallel",
            description: "parallel building mode.",
            flagPair: !0
        }, {
            name: "incremental",
            flag: "incremental",
            description: "incremental building mode.",
            flagPair: !0
        }, {
            name: "daemon",
            flag: "daemon",
            description: "building with daemon process.",
            flagPair: !0
        }, {
            name: "generateBuildProfile",
            flag: "generate-build-profile",
            description: "the generation of BuildProfile.ets files.",
            flagPair: !0
        }, {
            name: "analyze",
            flag: "analyze",
            description: "build analysis.",
            flagPair: !0
        }, {
            name: "analysisMode",
            flag: "--analyze=<analysisMode>",
            description: "Sets the build analysis mode: normal (default), advanced, false and ultrafine."
        } ]
    },
    command: [ {
        name: "version",
        description: "Shows the version of Hvigor."
    }, {
        name: "tasks",
        description: "Shows all available tasks of specific modules."
    }, {
        name: "taskTree",
        description: "Shows all available task trees of specific modules."
    }, {
        name: "prune",
        description: "Cleans up Hvigor cache files and removes unreferenced packages from store."
    }, {
        name: "collectCoverage",
        description: "Generates coverage statistics reports based on instrumentation test data."
    } ],
    help: {
        name: "",
        flag: "-h, --help",
        description: "Displays help information."
    },
    after: [ {
        position: "after",
        text: "\nExamples:\n  hvigor assembleApp  Do assembleApp task\n"
    }, {
        position: "after",
        text: "copyright 2023"
    } ]
};

!function(e) {
    var t = g && g.__createBinding || (Object.create ? function(e, t, r, n) {
        void 0 === n && (n = r);
        var o = Object.getOwnPropertyDescriptor(t, r);
        o && !("get" in o ? !t.__esModule : o.writable || o.configurable) || (o = {
            enumerable: !0,
            get: function() {
                return t[r];
            }
        }), Object.defineProperty(e, n, o);
    } : function(e, t, r, n) {
        void 0 === n && (n = r), e[n] = t[r];
    }), n = g && g.__setModuleDefault || (Object.create ? function(e, t) {
        Object.defineProperty(e, "default", {
            enumerable: !0,
            value: t
        });
    } : function(e, t) {
        e.default = t;
    }), o = g && g.__importStar || function(e) {
        if (e && e.__esModule) {
            return e;
        }
        var r = {};
        if (null != e) {
            for (var o in e) {
                "default" !== o && Object.prototype.hasOwnProperty.call(e, o) && t(r, e, o);
            }
        }
        return n(r, e), r;
    };
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.WORK_SPACE = e.HVIGOR_PROJECT_WRAPPER_HOME = e.HVIGOR_PNPM_STORE_PATH = e.HVIGOR_WRAPPER_PNPM_SCRIPT_PATH = e.HVIGOR_WRAPPER_TOOLS_HOME = e.HVIGOR_USER_HOME = e.META_DATA_JSON = e.DEFAULT_PACKAGE_JSON = e.DEFAULT_HVIGOR_CONFIG_JSON_FILE_NAME = e.PNPM_TOOL = e.DEFAULT_PROJECT_NODE_PATH = e.HVIGOR_PROJECT_ROOT_DIR = e.COMMAND_DESCRIPTION = e.CUR_HVIGOR_VERSION = e.HVIGOR_PACKAGE_NAME = void 0;
    const i = o(r), a = y, s = MU;
    e.HVIGOR_PACKAGE_NAME = l$, e.CUR_HVIGOR_VERSION = l$, e.COMMAND_DESCRIPTION = c$, 
    e.HVIGOR_PROJECT_ROOT_DIR = process.cwd(), e.DEFAULT_PROJECT_NODE_PATH = process.env.NODE_PATH, 
    e.PNPM_TOOL = (0, a.isWindows)() ? "pnpm.cmd" : "pnpm", e.DEFAULT_HVIGOR_CONFIG_JSON_FILE_NAME = "hvigor-config.json5", 
    e.DEFAULT_PACKAGE_JSON = "package.json", e.META_DATA_JSON = "metadata.json", e.HVIGOR_USER_HOME = (0, 
    s.getHvigorUserHomeCacheDir)(), e.HVIGOR_WRAPPER_TOOLS_HOME = i.resolve(e.HVIGOR_USER_HOME, "wrapper", "tools"), 
    e.HVIGOR_WRAPPER_PNPM_SCRIPT_PATH = i.resolve(e.HVIGOR_WRAPPER_TOOLS_HOME, "node_modules", ".bin", e.PNPM_TOOL), 
    e.HVIGOR_PNPM_STORE_PATH = i.resolve(e.HVIGOR_USER_HOME, "caches"), e.HVIGOR_PROJECT_WRAPPER_HOME = i.resolve(e.HVIGOR_PROJECT_ROOT_DIR, "hvigor"), 
    e.WORK_SPACE = "workspace";
}(u$), Object.defineProperty(a$, "__esModule", {
    value: !0
}), a$.globalHelpCommands = a$.globalVersionCommands = a$.GlobalExecute = void 0;

const f$ = s$, d$ = u$, p$ = c$;

function h$(e, t = 34, r = " ") {
    return e.padEnd(t, r);
}

a$.GlobalExecute = {
    version: e => {
        e.includes("--version") || e.includes("-v") ? (console.log(d$.CUR_HVIGOR_VERSION), 
        (0, f$.exit)(0)) : (console.log("hvigor", "[32mCLI version:", d$.CUR_HVIGOR_VERSION, "[0m"), 
        console.log("hvigor", "[32mLocal version:", d$.CUR_HVIGOR_VERSION || "Unknown", "[0m"), 
        (0, f$.exit)(0));
    },
    help: () => {
        console.log("Usage: ", p$.options.usage.flag, p$.options.usage.description, "\n"), 
        console.group("Options: ");
        const e = p$.options.version, t = p$.options.basicOptions;
        console.log(h$(e.flag), e.description), t.forEach(e => {
            console.log(h$(e.flag), e.description);
        });
        p$.options.otherOptions.forEach(e => {
            e.flagPair ? (console.log(h$("--".concat(e.flag)), "Enables ".concat(e.description)), 
            console.log(h$("--no-".concat(e.flag)), "Disables ".concat(e.description))) : console.log(h$(e.flag), e.description);
        });
        const r = p$.help;
        console.log(h$(r.flag), r.description, "\n"), console.groupEnd(), console.group("Commands: ");
        p$.command.forEach(e => {
            console.log(h$(e.name), e.description);
        }), console.groupEnd();
        const n = p$.after;
        null == n || n.forEach(e => {
            console.log(e.text);
        }), (0, f$.exit)(0);
    }
}, a$.globalVersionCommands = [ "-v", "--version", "version" ], a$.globalHelpCommands = [ "-h", "--help" ];

var v$ = {}, g$ = {}, m$ = g && g.__createBinding || (Object.create ? function(e, t, r, n) {
    void 0 === n && (n = r);
    var o = Object.getOwnPropertyDescriptor(t, r);
    o && !("get" in o ? !t.__esModule : o.writable || o.configurable) || (o = {
        enumerable: !0,
        get: function() {
            return t[r];
        }
    }), Object.defineProperty(e, n, o);
} : function(e, t, r, n) {
    void 0 === n && (n = r), e[n] = t[r];
}), y$ = g && g.__setModuleDefault || (Object.create ? function(e, t) {
    Object.defineProperty(e, "default", {
        enumerable: !0,
        value: t
    });
} : function(e, t) {
    e.default = t;
}), _$ = g && g.__importStar || function(e) {
    if (e && e.__esModule) {
        return e;
    }
    var t = {};
    if (null != e) {
        for (var r in e) {
            "default" !== r && Object.prototype.hasOwnProperty.call(e, r) && m$(t, e, r);
        }
    }
    return y$(t, e), t;
};

Object.defineProperty(g$, "__esModule", {
    value: !0
}), g$.checkSync = g$.unlockSync = g$.lockSync = void 0;

const E$ = _$(Bs), b$ = new Map;

function w$(e, t) {
    const r = A$(t);
    let n = 0;
    for (;n <= r.retries; ) {
        if (!S$(e)) {
            try {
                D$(e, r);
                break;
            } catch (e) {}
            n++;
        }
    }
    if (n >= r.retries) {
        throw new Error(`The registration information of the daemon cannot be obtained. Delete ${e}.lock and try again`);
    }
    !function(e, t) {
        var r;
        const n = `${e}.lock`, o = b$.get(e), i = () => {
            clearInterval(null == o ? void 0 : o.updateTimer), b$.delete(e), E$.removeSync(n);
        };
        if (void 0 === o) {
            return;
        }
        o.updateTimer = setInterval(() => {
            if (o.release) {
                clearInterval(o.updateTimer), E$.removeSync(n);
            } else {
                let e = o.lastUpdate + t.stale < Date.now();
                const r = new Date(Date.now());
                try {
                    if (o.mtime !== E$.statSync(n).mtime.getTime() || void 0 === o.mtime) {
                        return void i();
                    }
                } catch (t) {
                    ("ENOENT" === t.code || e) && i();
                }
                try {
                    E$.utimesSync(n, r, r);
                } catch (r) {
                    e = o.lastUpdate + t.stale < Date.now(), ("ENOENT" === r.code || e) && i();
                }
                o.mtime = r.getTime(), o.lastUpdate = Date.now();
            }
        }, t.update), null === (r = o.updateTimer) || void 0 === r || r.unref();
    }(e, r);
}

function D$(e, t) {
    const r = `${e}.lock`;
    try {
        E$.mkdirSync(r);
    } catch (n) {
        if ("EEXIST" !== n.code) {
            throw n;
        }
        try {
            if (!O$(E$.statSync(r), t)) {
                throw new Error(`Lock file ${r} has been held by other process.`);
            }
            E$.removeSync(r), w$(e, t);
        } catch (r) {
            if ("EONENT" !== r.code) {
                throw r;
            }
            w$(e, t);
        }
    }
    const n = new Date(Date.now() + 5);
    try {
        E$.utimesSync(r, n, n);
    } catch (e) {
        throw E$.removeSync(r), e;
    }
    b$.set(e, {
        mtime: n.getTime(),
        lastUpdate: Date.now(),
        option: t,
        lockPath: r,
        release: !1
    });
}

function S$(e, t) {
    const r = `${e}.lock`, n = A$(t);
    if (E$.existsSync(r)) {
        try {
            return !O$(E$.statSync(r), n);
        } catch (e) {
            if ("ENOENT" === e.code) {
                return !1;
            }
            throw e;
        }
    }
    return !1;
}

function A$(e) {
    const t = {
        stale: 1e4,
        retries: 0,
        update: 5e3,
        ...e
    };
    return t.stale = Math.max(t.stale || 2e3), t.update = Math.max(t.update, t.stale / 2), 
    t;
}

function O$(e, t) {
    return e.mtime.getTime() + t.stale < Date.now();
}

g$.lockSync = w$, g$.unlockSync = function(e) {
    const t = `${e}.lock`, r = b$.get(e);
    r && (b$.delete(e), r.release = !0, clearInterval(r.updateTimer), E$.removeSync(t));
}, g$.checkSync = S$;

var C$ = {}, x$ = {}, F$ = {}, M$ = {}, P$ = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(M$, "__esModule", {
    value: !0
}), M$.getHvigorUserHomeDir = void 0;

const I$ = P$(Bs), k$ = P$(a), R$ = P$(r), T$ = vc;

let j$ = !1;

var L$, N$, B$;

function U$() {
    if (L$) {
        return x$;
    }
    L$ = 1;
    var e = g && g.__createBinding || (Object.create ? function(e, t, r, n) {
        void 0 === n && (n = r);
        var o = Object.getOwnPropertyDescriptor(t, r);
        o && !("get" in o ? !t.__esModule : o.writable || o.configurable) || (o = {
            enumerable: !0,
            get: function() {
                return t[r];
            }
        }), Object.defineProperty(e, n, o);
    } : function(e, t, r, n) {
        void 0 === n && (n = r), e[n] = t[r];
    }), n = g && g.__setModuleDefault || (Object.create ? function(e, t) {
        Object.defineProperty(e, "default", {
            enumerable: !0,
            value: t
        });
    } : function(e, t) {
        e.default = t;
    }), o = g && g.__importStar || function(t) {
        if (t && t.__esModule) {
            return t;
        }
        var r = {};
        if (null != t) {
            for (var o in t) {
                "default" !== o && Object.prototype.hasOwnProperty.call(t, o) && e(r, t, o);
            }
        }
        return n(r, t), r;
    }, a = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(x$, "__esModule", {
        value: !0
    }), x$.isHvigorDependencyUseNpm = x$.isFileExists = x$.offlinePluginConversion = x$.executeCommand = x$.getNpmPath = x$.hasNpmPackInPaths = x$.BASE_NODE_VERSION = void 0;
    const s = i, u = a(t), l = a(Bs), c = o(r), f = CU, d = NU, p = H$(), h = vc, v = F$;
    x$.BASE_NODE_VERSION = "16.0.0";
    const m = "hvigor.dependency.useNpm";
    return x$.hasNpmPackInPaths = function(e, t) {
        try {
            return require.resolve(e, {
                paths: [ ...t ]
            }), !0;
        } catch (e) {
            return !1;
        }
    }, x$.getNpmPath = function() {
        const e = process.execPath;
        return c.join(c.dirname(e), f.NPM_TOOL);
    }, x$.executeCommand = function(e, t, r) {
        if (0 !== (0, s.spawnSync)(e, t, r).status) {
            let r = "See above for details.";
            e.includes(" ") && (r = "Space is not supported in HVIGOR_USER_HOME. Remove the space in HVIGOR_USER_HOME to fix the issue."), 
            (0, h.logFormatedErrorAndExit)("00308002", `${e} ${t} execute failed.`, [ r ]);
        }
    }, x$.offlinePluginConversion = function(e, t) {
        return t.startsWith("file:") || t.endsWith(".tgz") ? c.resolve(e, f.HVIGOR, t.replace("file:", "")) : t;
    }, x$.isFileExists = function(e) {
        return u.default.existsSync(e) && u.default.statSync(e).isFile();
    }, x$.isHvigorDependencyUseNpm = function() {
        var e, t, r;
        const n = c.resolve(v.HVIGOR_USER_HOME, f.DEFAULT_HVIGOR_CONFIG_JSON_FILE_NAME);
        let o;
        l.default.existsSync(n) && (o = (0, d.parseJsonFile)(n));
        const i = null !== (r = null !== (t = null === (e = (0, p.readProjectHvigorConfig)()) || void 0 === e ? void 0 : e.properties) && void 0 !== t ? t : null == o ? void 0 : o.properties) && void 0 !== r ? r : void 0;
        return !(!i || !i[m]) && i[m];
    }, x$;
}

function z$() {
    return N$ || (N$ = 1, function(e) {
        var n = g && g.__createBinding || (Object.create ? function(e, t, r, n) {
            void 0 === n && (n = r);
            var o = Object.getOwnPropertyDescriptor(t, r);
            o && !("get" in o ? !t.__esModule : o.writable || o.configurable) || (o = {
                enumerable: !0,
                get: function() {
                    return t[r];
                }
            }), Object.defineProperty(e, n, o);
        } : function(e, t, r, n) {
            void 0 === n && (n = r), e[n] = t[r];
        }), o = g && g.__setModuleDefault || (Object.create ? function(e, t) {
            Object.defineProperty(e, "default", {
                enumerable: !0,
                value: t
            });
        } : function(e, t) {
            e.default = t;
        }), s = g && g.__importStar || function(e) {
            if (e && e.__esModule) {
                return e;
            }
            var t = {};
            if (null != e) {
                for (var r in e) {
                    "default" !== r && Object.prototype.hasOwnProperty.call(e, r) && n(t, e, r);
                }
            }
            return o(t, e), t;
        }, u = g && g.__importDefault || function(e) {
            return e && e.__esModule ? e : {
                default: e
            };
        };
        Object.defineProperty(e, "__esModule", {
            value: !0
        }), e.executeInstallPnpm = e.isPnpmInstalled = e.environmentHandler = e.checkNpmConifg = e.PNPM_VERSION = void 0;
        const l = i, c = s(t), f = u(a), d = s(r), p = CU, h = MU, v = H$(), m = vc, y = U$();
        function _() {
            (0, m.logFormatedError)("00303137", "The hvigor depends on the npmrc file. No npmrc file is matched in the current user folder. Configure the npmrc file first.", [ "Configure the .npmrc file in the user directory." ]);
        }
        e.PNPM_VERSION = "8.13.1", e.checkNpmConifg = function() {
            const e = d.resolve(process.cwd(), ".npmrc"), t = d.resolve(f.default.homedir(), ".npmrc");
            if (process.env.npm_config_registry && process.env["npm_config_@ohos:registry"]) {
                return;
            }
            const r = (0, v.readProjectHvigorConfig)();
            if (!(null == r ? void 0 : r.dependencies) || 0 === Object.entries(null == r ? void 0 : r.dependencies).length) {
                return;
            }
            if ((0, y.isFileExists)(e) || (0, y.isFileExists)(t)) {
                return;
            }
            const n = (0, y.getNpmPath)(), o = (0, l.spawnSync)(n, [ "config", "get", "prefix" ], {
                cwd: process.cwd()
            });
            if (0 !== o.status || !o.stdout) {
                return void _();
            }
            const i = d.resolve(`${o.stdout}`.replace(/[\r\n]/gi, ""), ".npmrc");
            (0, y.isFileExists)(i) || _();
        }, e.environmentHandler = function() {
            process.env["npm_config_update-notifier"] = "false", process.env["npm_config_auto-install-peers"] = "false";
        };
        const E = (0, h.getHvigorUserHomeCacheDir)(), b = d.resolve(E, "wrapper", "tools"), w = d.resolve(b, "node_modules", ".bin", p.PNPM_TOOL);
        e.isPnpmInstalled = function() {
            return !!c.existsSync(w) && (0, y.hasNpmPackInPaths)("pnpm", [ b ]);
        }, e.executeInstallPnpm = function() {
            (0, m.logInfo)(`Installing pnpm@${e.PNPM_VERSION}...`);
            const t = (0, y.getNpmPath)();
            !function() {
                const t = d.resolve(b, p.DEFAULT_PACKAGE_JSON);
                try {
                    c.existsSync(b) || c.mkdirSync(b, {
                        recursive: !0
                    });
                    const r = {
                        dependencies: {}
                    };
                    r.dependencies[p.PNPM] = e.PNPM_VERSION, c.writeFileSync(t, JSON.stringify(r));
                } catch (e) {
                    (0, m.logFormatedErrorAndExit)("00307001", `EPERM: operation not permitted,create ${t} failed.`, [ "Check whether you have the permission to write files." ]);
                }
            }(), (0, y.executeCommand)(t, [ "install", "pnpm" ], {
                cwd: b,
                stdio: [ "inherit", "inherit", "inherit" ],
                env: process.env
            }), (0, m.logInfo)("Pnpm install success.");
        };
    }(C$)), C$;
}

function H$() {
    if (B$) {
        return v$;
    }
    B$ = 1;
    var e = g && g.__createBinding || (Object.create ? function(e, t, r, n) {
        void 0 === n && (n = r);
        var o = Object.getOwnPropertyDescriptor(t, r);
        o && !("get" in o ? !t.__esModule : o.writable || o.configurable) || (o = {
            enumerable: !0,
            get: function() {
                return t[r];
            }
        }), Object.defineProperty(e, n, o);
    } : function(e, t, r, n) {
        void 0 === n && (n = r), e[n] = t[r];
    }), o = g && g.__setModuleDefault || (Object.create ? function(e, t) {
        Object.defineProperty(e, "default", {
            enumerable: !0,
            value: t
        });
    } : function(e, t) {
        e.default = t;
    }), i = g && g.__importStar || function(t) {
        if (t && t.__esModule) {
            return t;
        }
        var r = {};
        if (null != t) {
            for (var n in t) {
                "default" !== n && Object.prototype.hasOwnProperty.call(t, n) && e(r, t, n);
            }
        }
        return o(r, t), r;
    }, s = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(v$, "__esModule", {
        value: !0
    }), v$.readProjectHvigorConfig = v$.linkHvigorToWorkspace = v$.initProjectWorkSpace = void 0;
    const u = i(t), l = s(Bs), c = s(a), f = i(r), d = s(n), p = pc, h = g$, v = CU, m = FU, _ = y, E = NU, b = vc, w = z$(), D = U$(), S = F$, A = s$, O = l$;
    let C, x, F;
    const M = d.default.version.slice(1);
    v$.initProjectWorkSpace = function() {
        if (C = j(), F = f.resolve(S.HVIGOR_USER_HOME, v.PROJECT_CACHES, (0, _.hash)(f.resolve(d.default.cwd())), v.WORK_SPACE), 
        x = function() {
            const e = f.resolve(F, v.DEFAULT_PACKAGE_JSON);
            return u.existsSync(e) ? (0, E.parseJsonFile)(e) : {
                dependencies: {}
            };
        }(), !function() {
            function e(e) {
                const t = null == e ? void 0 : e.dependencies;
                return void 0 === t ? 0 : Object.getOwnPropertyNames(t).length;
            }
            const t = e(C), r = e(x);
            if (t !== r) {
                return !1;
            }
            for (const e in null == C ? void 0 : C.dependencies) {
                if (!(0, D.hasNpmPackInPaths)(e, [ F ]) || !R(e, C, x)) {
                    return !1;
                }
            }
            return !0;
        }()) {
            try {
                (0, w.checkNpmConifg)(), function() {
                    var e;
                    (0, b.logInfo)("Installing dependencies...");
                    const t = f.resolve(F, ".pnpmfile.js");
                    l.default.existsSync(t) && l.default.rmSync(t, {
                        force: !0
                    });
                    for (const [t, r] of Object.entries(null !== (e = null == C ? void 0 : C.dependencies) && void 0 !== e ? e : {})) {
                        r && (C.dependencies[t] = (0, D.offlinePluginConversion)(d.default.cwd(), r));
                    }
                    const r = {
                        dependencies: {
                            ...C.dependencies
                        }
                    };
                    try {
                        u.mkdirSync(F, {
                            recursive: !0
                        });
                        const e = f.resolve(F, v.DEFAULT_PACKAGE_JSON);
                        u.writeFileSync(e, JSON.stringify(r));
                    } catch (e) {
                        (0, b.logErrorAndExit)(e);
                    }
                    (function() {
                        const e = [ "install" ];
                        (0, _.isCI)() && e.push("--no-frozen-lockfile");
                        l.default.existsSync(f.resolve(m.HVIGOR_PROJECT_ROOT_DIR, ".npmrc")) && (d.default.env.npm_config_userconfig = function(e) {
                            const t = T();
                            try {
                                let r = "";
                                const n = f.resolve(c.default.homedir(), ".npmrc");
                                l.default.existsSync(n) && (r = l.default.readFileSync(n, "utf-8"));
                                const o = l.default.readFileSync(e, "utf-8"), i = `${r}${c.default.EOL}${o}`;
                                l.default.ensureFileSync(t), l.default.writeFileSync(t, i);
                            } catch (e) {
                                (0, b.logErrorAndExit)(e);
                            }
                            return t;
                        }(f.resolve(m.HVIGOR_PROJECT_ROOT_DIR, ".npmrc")));
                        const t = {
                            cwd: F,
                            stdio: [ "inherit", "inherit", "inherit" ],
                            env: d.default.env
                        };
                        if ((0, D.isHvigorDependencyUseNpm)() || (0, p.lt)(M, D.BASE_NODE_VERSION)) {
                            const t = f.resolve(F, "node_modules/@ohos/hvigor"), r = f.resolve(F, "node_modules/@ohos/hvigor-ohos-plugin");
                            u.existsSync(t) && u.rmSync(t, {
                                recursive: !0,
                                force: !0
                            }), u.existsSync(r) && u.rmSync(r, {
                                recursive: !0,
                                force: !0
                            }), (0, D.executeCommand)(v.NPM_TOOL, e, {
                                cwd: F
                            }), I(F);
                        } else {
                            !function() {
                                try {
                                    const e = T();
                                    let t = [];
                                    if (l.default.existsSync(e)) {
                                        t = l.default.readFileSync(e, "utf-8").split(c.default.EOL), t.every(e => !e.startsWith("store-dir")) && t.push(`store-dir=${S.HVIGOR_PNPM_STORE_PATH}`);
                                    } else {
                                        t.push(`store-dir=${S.HVIGOR_PNPM_STORE_PATH}`);
                                    }
                                    (0, h.lockSync)(e, {
                                        retries: 100,
                                        update: 1e3
                                    }), l.default.ensureFileSync(e), l.default.writeFileSync(e, t.join(c.default.EOL)), 
                                    (0, h.unlockSync)(e);
                                } catch (e) {
                                    (0, b.logErrorAndExit)(e);
                                }
                            }(), function() {
                                const e = f.resolve(F, "node_modules", ".modules.yaml"), t = T(), r = f.resolve(F, "..", ".npmrc");
                                try {
                                    if (l.default.existsSync(r) && l.default.removeSync(r), !l.default.existsSync(e) || !l.default.existsSync(t)) {
                                        return;
                                    }
                                    const n = l.default.readFileSync(e, "utf-8"), o = l.default.readFileSync(t, "utf-8"), i = n.match(/^storeDir\s*:\s*(.+)$/m), a = o.match(/^store-dir\s*=\s*(.+)$/m);
                                    if (!i || !a) {
                                        return;
                                    }
                                    const s = i[1].trim(), u = a[1].trim();
                                    f.resolve(f.dirname(s)) !== f.resolve(u) && l.default.removeSync(e);
                                } catch (t) {
                                    (0, b.logInfo)(`Failed to process .modules.yaml at ${e}. Delete again. \n Reason: ${t}`), 
                                    l.default.existsSync(e) && l.default.removeSync(e);
                                }
                            }(), (0, D.executeCommand)(S.HVIGOR_WRAPPER_PNPM_SCRIPT_PATH, e, t);
                        }
                    })(), (0, b.logInfo)("Hvigor install success.");
                }();
            } catch (e) {
                e instanceof Error && (0, b.logError)("Got Error when installing hvigor or its dependencies, will clean work space: " + e.message), 
                function() {
                    if ((0, b.logInfo)("Hvigor cleaning..."), !u.existsSync(F)) {
                        return;
                    }
                    const e = u.readdirSync(F);
                    if (!e || 0 === e.length) {
                        return;
                    }
                    const t = f.resolve(F, "node_modules", "@ohos", "hvigor", "bin", "hvigor.js");
                    u.existsSync(t) && (0, D.executeCommand)(d.default.argv[0], [ t, "--stop-daemon" ], {});
                    try {
                        e.forEach(e => {
                            u.rmSync(f.resolve(F, e), {
                                recursive: !0
                            });
                        });
                    } catch (e) {
                        (0, b.logErrorAndExit)(`The hvigor build tool cannot be installed. Please manually clear the workspace directory and synchronize the project again.\n\n      Workspace Path: ${F}.`);
                    }
                }();
            }
        }
        return I(F), F;
    };
    const P = "win32" === d.default.platform || "Windows_NT" === c.default.type();
    function I(e) {
        const t = f.resolve(__dirname, ".."), r = f.resolve(e, "node_modules", "@ohos"), n = P ? "junction" : "dir";
        try {
            l.default.ensureDirSync(r), (null == C ? void 0 : C.dependencies["@ohos/hvigor"]) || k(f.resolve(r, "hvigor"), f.resolve(t, "hvigor"), n), 
            (null == C ? void 0 : C.dependencies["@ohos/hvigor-ohos-plugin"]) || k(f.resolve(r, "hvigor-ohos-plugin"), f.resolve(t, "hvigor-ohos-plugin"), n), 
            (null == C ? void 0 : C.dependencies["@ohos/cangjie-build-support"]) || function(e, t, r) {
                const n = d.default.env.CANGJIE_BUILD_SUPPORT_PATH;
                if (void 0 !== n) {
                    const e = f.normalize(n);
                    if (u.existsSync(e)) {
                        return void k(f.resolve(t, "cangjie-build-support"), e, r);
                    }
                }
                const o = f.resolve(e, "cangjie-build-support");
                if (u.existsSync(o)) {
                    return void k(f.resolve(t, "cangjie-build-support"), o, r);
                }
                const i = d.default.env.DEVECO_CANGJIE_PATH;
                if (void 0 !== i) {
                    const e = f.normalize(i), n = f.resolve(e, "build-tools", "tools", "hvigor", "cangjie-build-support");
                    if (u.existsSync(n)) {
                        return void k(f.resolve(t, "cangjie-build-support"), n, r);
                    }
                }
                const a = d.default.env.DEVECO_HVIGOR_CANGJIE_PLUGIN;
                if (void 0 !== a) {
                    const e = f.normalize(a);
                    u.existsSync(e) && k(f.resolve(t, "cangjie-build-support"), e, r);
                }
            }(t, r, n);
        } catch (e) {
            (0, b.logErrorAndExit)(e);
        }
    }
    function k(e, t, r) {
        try {
            if (!u.existsSync(e)) {
                return void u.symlinkSync(t, e, r);
            }
            const n = f.resolve(u.readlinkSync(e));
            if (!u.lstatSync(e).isSymbolicLink() || n !== t) {
                return u.rmSync(e, {
                    recursive: !0,
                    force: !0
                }), void u.symlinkSync(t, e, r);
            }
            (0, E.parseJsonFile)(f.resolve(n, "package.json")).version !== O && (u.rmSync(e, {
                recursive: !0,
                force: !0
            }), u.symlinkSync(t, e, r));
        } catch (n) {
            u.rmSync(e, {
                recursive: !0,
                force: !0
            }), u.symlinkSync(t, e, r);
        }
    }
    function R(e, t, r) {
        return void 0 !== r.dependencies && (0, D.offlinePluginConversion)(d.default.cwd(), t.dependencies[e]) === f.normalize(r.dependencies[e]);
    }
    function T() {
        return f.resolve(F, ".npmrc");
    }
    function j() {
        var e;
        const t = f.resolve(m.HVIGOR_PROJECT_WRAPPER_HOME, v.DEFAULT_HVIGOR_CONFIG_JSON_FILE_NAME);
        let r;
        u.existsSync(t) || (0, b.logFormatedErrorAndExit)("00304004", `Hvigor config file ${t} does not exist.`, [ "Check whether the hvigor-config.json5 file exists." ]);
        try {
            r = (0, E.parseJsonFile)(t), r.dependencies = null !== (e = r.dependencies) && void 0 !== e ? e : {};
        } catch (e) {
            if (e instanceof Error) {
                let t = `${e.message}`;
                d.default.argv.includes("--stacktrace") && e.stack && (t += `${e.stack}`);
                const r = [ "Correct the syntax error as indicated above in the hvigor-config.json5 file." ];
                (0, b.logFormatedError)("00303236", t, r), (0, A.exit)(-1);
            }
        }
        return r;
    }
    return v$.linkHvigorToWorkspace = I, v$.readProjectHvigorConfig = j, v$;
}

M$.getHvigorUserHomeDir = function() {
    const e = R$.default.resolve(k$.default.homedir(), ".hvigor"), t = process.env.HVIGOR_USER_HOME;
    return void 0 === t ? e : R$.default.isAbsolute(t) ? I$.default.existsSync(t) && I$.default.statSync(t).isFile() ? ((0, 
    T$.logInfo)(`File already exists: ${t}`), e) : (I$.default.ensureDirSync(t), t) : (j$ || ((0, 
    T$.logInfo)(`Invalid custom userhome hvigor data dir:${t}`), j$ = !0), e);
}, function(e) {
    var t = g && g.__importDefault || function(e) {
        return e && e.__esModule ? e : {
            default: e
        };
    };
    Object.defineProperty(e, "__esModule", {
        value: !0
    }), e.HVIGOR_PROJECT_WRAPPER_HOME = e.HVIGOR_PROJECT_ROOT_DIR = e.HVIGOR_PNPM_STORE_PATH = e.HVIGOR_WRAPPER_PNPM_SCRIPT_PATH = e.HVIGOR_WRAPPER_TOOLS_HOME = e.HVIGOR_USER_HOME = void 0;
    const n = t(r), o = CU, i = M$;
    e.HVIGOR_USER_HOME = (0, i.getHvigorUserHomeDir)(), e.HVIGOR_WRAPPER_TOOLS_HOME = n.default.resolve(e.HVIGOR_USER_HOME, "wrapper", "tools"), 
    e.HVIGOR_WRAPPER_PNPM_SCRIPT_PATH = n.default.resolve(e.HVIGOR_WRAPPER_TOOLS_HOME, "node_modules", ".bin", o.PNPM_TOOL), 
    e.HVIGOR_PNPM_STORE_PATH = n.default.resolve(e.HVIGOR_USER_HOME, "caches"), e.HVIGOR_PROJECT_ROOT_DIR = process.cwd(), 
    e.HVIGOR_PROJECT_WRAPPER_HOME = n.default.resolve(e.HVIGOR_PROJECT_ROOT_DIR, o.HVIGOR);
}(F$);

var $$ = g && g.__importDefault || function(e) {
    return e && e.__esModule ? e : {
        default: e
    };
};

Object.defineProperty(m, "__esModule", {
    value: !0
});

const G$ = $$(t), W$ = $$(r), V$ = $$(n), K$ = y, q$ = pc, J$ = hc, X$ = a$, Z$ = H$(), Y$ = z$(), Q$ = U$();

!function() {
    (0, Y$.environmentHandler)(), function() {
        const e = V$.default.argv.slice(2);
        e.filter(e => X$.globalVersionCommands.includes(e)).length > 0 && X$.GlobalExecute.version(e.toString()), 
        e.filter(e => X$.globalHelpCommands.includes(e)).length > 0 && X$.GlobalExecute.help();
    }(), (0, q$.gte)(V$.default.version.slice(1), Q$.BASE_NODE_VERSION) && !(0, Q$.isHvigorDependencyUseNpm)() && ((0, 
    Y$.isPnpmInstalled)() || ((0, Y$.checkNpmConifg)(), (0, Y$.executeInstallPnpm)()));
    const e = W$.default.resolve(__dirname, "../../ohpm/bin/", (0, K$.isWindows)() ? "ohpm.bat" : "ohpm");
    G$.default.existsSync(e) && (V$.default.env.ohpmBin = e);
    const t = (0, Z$.initProjectWorkSpace)();
    (0, J$.executeBuild)(t);
}(), module.exports = m;