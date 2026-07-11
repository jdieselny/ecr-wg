import crypto from 'node:crypto';
import { execFileSync, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const crateRoot = path.resolve(__dirname, '..');

function resolveEmiliaRoot() {
  const candidates = [
    process.env.EP_EMILIA_PROTOCOL_ROOT,
    path.resolve(crateRoot, '..', '..', '..', 'emilia-protocol'), // ecr-wg sibling
    path.resolve(crateRoot, '..', '..', 'emilia-protocol'),
    'C:/Users/jkintzele/Documents/emilia-protocol', // legacy local default
  ].filter(Boolean);
  for (const c of candidates) {
    const bundle = path.join(c, 'conformance', 'clean-room', 'bundle.v1.json');
    if (fs.existsSync(bundle)) return c;
  }
  console.error('emilia-protocol root not found. Set EP_EMILIA_PROTOCOL_ROOT or clone beside ecr-wg.');
  process.exit(2);
}

const ROOT = resolveEmiliaRoot();
const defaultBin = path.join(
  crateRoot,
  'target',
  'release',
  process.platform === 'win32' ? 'conformance.exe' : 'conformance',
);
const BIN = process.argv[2] || defaultBin;
if (!fs.existsSync(BIN)) {
  console.error('conformance binary not found:', BIN);
  console.error('Build with: cargo build --release --bin conformance');
  process.exit(2);
}
const BUNDLE = JSON.parse(fs.readFileSync(path.join(ROOT, 'conformance/clean-room/bundle.v1.json'), 'utf8'));
const PRIMARY_FIELDS = [
  'document', 'signoff', 'quorum', 'revocation', 'time_attestation',
  'trust_receipt', 'provenance_chain', 'evidence_record', 'canonicalization',
  'currency', 'initiator_attestation', 'consumption_proof', 'witness_quorum',
  'timestamp_proof',
];
const destructiveValues = [null, {}, [], '', true, 9007199254740992];
function clone(v){ return structuredClone(v); }
const cases = [];
const caseSuites = new Map();
const expectations = new Map();
for (const suiteRef of BUNDLE.suites) {
  const suite = JSON.parse(fs.readFileSync(path.join(ROOT, suiteRef.path), 'utf8'));
  const selected = [];
  const positive = suite.vectors.find((vector) => vector.expect?.valid === true);
  const negative = suite.vectors.find((vector) => vector.expect?.valid === false);
  if (positive) selected.push(positive);
  if (negative && negative !== positive) selected.push(negative);
  for (const source of selected) {
    const prefix = `${path.basename(suiteRef.path, '.json')}_${source.id}`.replace(/[^a-zA-Z0-9_.-]/g, '_');
    const primary = PRIMARY_FIELDS.find((field) => Object.hasOwn(source, field));
    if (!primary) throw new Error('no primary for ' + suiteRef.path);
    // only the hostile-type cases that caused 12 panics + keep raw cases separate
    for (let i = 0; i < destructiveValues.length; i += 1) {
      const hostile = clone(source);
      hostile.id = `${prefix}__type_${i}`;
      hostile[primary] = clone(destructiveValues[i]);
      cases.push(hostile);
      caseSuites.set(hostile.id, suite.suite);
      expectations.set(hostile.id, { kind: 'reject' });
    }
  }
}
let deep = { leaf: true };
for (let i = 0; i < 66; i += 1) deep = { nested: deep };
const rawParserCases = [
  { id: 'truncated-json', bytes: Buffer.from('{"suite":"EP-RECEIPT-v1","vectors":[') },
  { id: 'duplicate-root-member', bytes: Buffer.from('{"suite":"EP-RECEIPT-v1","vectors":[],"vectors":[]}') },
  { id: 'duplicate-vector-member', bytes: Buffer.from('{"suite":"EP-RECEIPT-v1","vectors":[{"id":"a","id":"b"}]}') },
  { id: 'unpaired-surrogate', bytes: Buffer.from('{"suite":"EP-RECEIPT-v1","vectors":[{"id":"\\ud800"}]}') },
  { id: 'over-depth', bytes: Buffer.from(JSON.stringify({ suite: 'EP-RECEIPT-v1', vectors: [{ id: 'deep', document: deep }] })) },
  { id: 'invalid-utf8', bytes: Buffer.concat([Buffer.from('{"suite":"EP-RECEIPT-v1","vectors":[],"x":"'), Buffer.from([0xc3, 0x28]), Buffer.from('"}')]) },
];
const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'ep-hostility-rust-'));
const findings = [];
for (const rawCase of rawParserCases) {
  const rawPath = path.join(dir, `${rawCase.id}.json`);
  fs.writeFileSync(rawPath, rawCase.bytes);
  const result = spawnSync(BIN, [rawPath], { encoding: 'utf8', timeout: 60000, maxBuffer: 64*1024*1024, stdio: ['ignore','pipe','pipe'] });
  if (result.status === 0) findings.push({ id: rawCase.id, reason: 'malformed_raw_json_accepted', status: result.status, stderr: (result.stderr||'').slice(0,200) });
  else if (result.signal || /panicked at/.test(result.stderr || '')) findings.push({ id: rawCase.id, reason: 'runner_crash', signal: result.signal, stderr: (result.stderr||'').slice(0,300) });
  else console.log('RAW OK', rawCase.id, 'exit', result.status, (result.stderr||'').trim().slice(0,80));
}
// suite dispatch for type hostilities (only canonicalization ones that used to panic + all for safety)
const bySuite = new Map();
for (const c of cases) {
  const s = caseSuites.get(c.id);
  if (!bySuite.has(s)) bySuite.set(s, []);
  bySuite.get(s).push(c);
}
for (const [suite, suiteCases] of bySuite) {
  // batch first
  const suitePath = path.join(dir, `suite-${suite.replace(/[^a-zA-Z0-9_.-]/g,'_')}.json`);
  fs.writeFileSync(suitePath, JSON.stringify({ suite, vectors: suiteCases }) + '\n');
  let batchOk = false;
  try {
    const stdout = execFileSync(BIN, [suitePath], { encoding: 'utf8', timeout: 180000, maxBuffer: 64*1024*1024, stdio: ['ignore','pipe','pipe'] });
    const parsed = JSON.parse(stdout);
    if (!Array.isArray(parsed) || parsed.length !== suiteCases.length) throw new Error('bad count ' + parsed?.length);
    for (const r of parsed) {
      if (typeof r.valid !== 'boolean') findings.push({ id: r.id, reason: 'malformed_result' });
      else if (r.valid !== false && expectations.get(r.id)?.kind === 'reject') findings.push({ id: r.id, reason: 'hostile_input_accepted', valid: r.valid });
    }
    batchOk = true;
    console.log('SUITE OK', suite, suiteCases.length);
  } catch (e) {
    console.log('SUITE BATCH FAIL', suite, String(e.message).slice(0,120));
  }
  if (!batchOk) {
    for (const hostile of suiteCases) {
      const singlePath = path.join(dir, `case-${hostile.id}.json`);
      fs.writeFileSync(singlePath, JSON.stringify({ suite, vectors: [hostile] }) + '\n');
      const result = spawnSync(BIN, [singlePath], { encoding: 'utf8', timeout: 60000, maxBuffer: 64*1024*1024, stdio: ['ignore','pipe','pipe'] });
      if (result.signal || /panicked at/.test(result.stderr || '')) {
        findings.push({ id: hostile.id, reason: 'runner_crash', stderr: (result.stderr||'').slice(0,300) });
      } else if (result.status !== 0) {
        findings.push({ id: hostile.id, reason: 'nonzero_exit_on_structured', status: result.status, stderr: (result.stderr||'').slice(0,200) });
      } else {
        try {
          const parsed = JSON.parse(result.stdout);
          const r = parsed[0];
          if (!r || r.valid !== false) findings.push({ id: hostile.id, reason: 'hostile_input_accepted', out: result.stdout.slice(0,200) });
        } catch {
          findings.push({ id: hostile.id, reason: 'invalid_json_out', out: (result.stdout||'').slice(0,200) });
        }
      }
    }
  }
}
fs.rmSync(dir, { recursive: true, force: true });
console.log('FINDINGS', findings.length);
for (const f of findings) console.log(JSON.stringify(f));
if (findings.length) process.exit(1);
console.log('HOSTILITY LOCAL: PASS (raw refusals + structured type rejects, no panics, no fail-open)');
