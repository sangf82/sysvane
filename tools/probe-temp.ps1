# Read-only probe: reads AMD CPU temperature registers through PawnIO. Needs an elevated shell.
# Repeats the registers and decoding of core/src/cputemp.rs, for use without the core.
# Run from an administrator terminal: tools\probe-temp.cmd
param([string]$Module = (Join-Path $PSScriptRoot '..\core\pawnio\AMDFamily17.bin'))

Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class P {
  const string Dll = @"C:\Program Files\PawnIO\PawnIOLib.dll";
  [DllImport(Dll)] public static extern int pawnio_open(out IntPtr h);
  [DllImport(Dll)] public static extern int pawnio_load(IntPtr h, byte[] blob, ulong size);
  [DllImport(Dll)] public static extern int pawnio_execute(IntPtr h, [MarshalAs(UnmanagedType.LPStr)] string name, ulong[] i, ulong isz, ulong[] o, ulong osz, out ulong ret);
  [DllImport(Dll)] public static extern int pawnio_close(IntPtr h);
}
'@

$admin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
"admin: $admin"
$h = [IntPtr]::Zero
$r = [P]::pawnio_open([ref]$h)
'open: 0x{0:X8}' -f $r
if ($r -ne 0) { return }
try {
  $blob = [IO.File]::ReadAllBytes((Resolve-Path $Module))
  $r = [P]::pawnio_load($h, $blob, [uint64]$blob.Length)
  'load: 0x{0:X8}' -f $r
  if ($r -ne 0) { return }
  foreach ($reg in 0x59800, 0x59954, 0x59958, 0x59B08, 0x59B0C) {
    $out = New-Object 'uint64[]' 1
    $ret = [uint64]0
    $r = [P]::pawnio_execute($h, 'ioctl_read_smn', [uint64[]]@([uint64]$reg), 1, $out, 1, [ref]$ret)
    $v = [uint32]($out[0] -band 0xFFFFFFFF)
    'smn 0x{0:X5}: hr=0x{1:X8} val=0x{2:X8}' -f $reg, $r, $v
    if ($r -ne 0) { continue }
    if ($reg -eq 0x59800) {
      $t = ($v -shr 21) * 0.125
      if ($v -band 0x80000) { $t -= 49 }   # range-select bit: the scale is shifted by 49 C
      "  Tctl = $t C"
    } elseif ($v -band 0x800) {            # bit 11: the CCD reading is valid
      "  CCD  = $(($v -band 0x7FF) * 0.125 - 49) C"
    }
  }
} finally {
  [void][P]::pawnio_close($h)
}
