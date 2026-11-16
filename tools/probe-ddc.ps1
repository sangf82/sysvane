# Read-only DDC/CI probe: prints current and maximum of a few VCP codes on every monitor.
# Never writes. Usage: powershell -NoProfile -File tools\probe-ddc.ps1 [-Codes 0x14,0xB6]
param([int[]]$Codes = @(0x10, 0x12, 0x14, 0x0C, 0x16, 0x18, 0x1A, 0xB6, 0xDC))

Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
public static class Ddc {
    [StructLayout(LayoutKind.Sequential, CharSet = CharSet.Unicode)]
    public struct PHYSICAL_MONITOR { public IntPtr h; [MarshalAs(UnmanagedType.ByValTStr, SizeConst = 128)] public string desc; }
    delegate bool EnumProc(IntPtr m, IntPtr dc, IntPtr r, IntPtr d);
    [DllImport("user32.dll")] static extern bool EnumDisplayMonitors(IntPtr dc, IntPtr clip, EnumProc f, IntPtr d);
    [DllImport("dxva2.dll", SetLastError = true)] static extern bool GetNumberOfPhysicalMonitorsFromHMONITOR(IntPtr m, out uint n);
    [DllImport("dxva2.dll", SetLastError = true)] static extern bool GetPhysicalMonitorsFromHMONITOR(IntPtr m, uint n, [Out] PHYSICAL_MONITOR[] a);
    [DllImport("dxva2.dll")] static extern bool GetVCPFeatureAndVCPFeatureReply(IntPtr h, byte code, out uint kind, out uint cur, out uint max);
    [DllImport("dxva2.dll")] static extern bool DestroyPhysicalMonitor(IntPtr h);
    public static List<string> Read(int[] codes) {
        var hms = new List<IntPtr>();
        EnumDisplayMonitors(IntPtr.Zero, IntPtr.Zero, (m, dc, r, d) => { hms.Add(m); return true; }, IntPtr.Zero);
        var outp = new List<string>();
        for (int i = 0; i < hms.Count; i++) {
            uint n; if (!GetNumberOfPhysicalMonitorsFromHMONITOR(hms[i], out n) || n == 0) { outp.Add("monitor " + i + ": no physical monitor (" + Marshal.GetLastWin32Error() + ")"); continue; }
            var a = new PHYSICAL_MONITOR[n]; if (!GetPhysicalMonitorsFromHMONITOR(hms[i], n, a)) { outp.Add("monitor " + i + ": handles refused (" + Marshal.GetLastWin32Error() + ")"); continue; }
            foreach (var p in a) {
                var line = "monitor " + i + ":";
                foreach (var c in codes) {
                    uint k, cur, max;
                    line += GetVCPFeatureAndVCPFeatureReply(p.h, (byte)c, out k, out cur, out max)
                        ? string.Format(" 0x{0:X2}={1}/{2}", c, cur, max) : string.Format(" 0x{0:X2}=n/a", c);
                }
                outp.Add(line);
                DestroyPhysicalMonitor(p.h);
            }
        }
        outp.Insert(0, hms.Count + " display(s)");
        return outp;
    }
}
'@
[Ddc]::Read($Codes)
