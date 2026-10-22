# Use the Unicode Shell Link interface; WScript.Shell can reject non-ASCII targets.
if(-not ('IELTSStudioShortcut' -as [type])){
 Add-Type -TypeDefinition @'
using System;
using System.IO;
using System.Text;
using System.Runtime.InteropServices;
using System.Runtime.InteropServices.ComTypes;
[ComImport, Guid("00021401-0000-0000-C000-000000000046")]
class IELTSStudioShellLink {}
[ComImport, InterfaceType(ComInterfaceType.InterfaceIsIUnknown), Guid("000214F9-0000-0000-C000-000000000046")]
interface IELTSStudioShellLinkW {
 void GetPath([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder path,int capacity,IntPtr data,int flags);
 void GetIDList(out IntPtr list);
 void SetIDList(IntPtr list);
 void GetDescription([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder text,int capacity);
 void SetDescription([MarshalAs(UnmanagedType.LPWStr)] string text);
 void GetWorkingDirectory([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder path,int capacity);
 void SetWorkingDirectory([MarshalAs(UnmanagedType.LPWStr)] string path);
 void GetArguments([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder text,int capacity);
 void SetArguments([MarshalAs(UnmanagedType.LPWStr)] string text);
 void GetHotkey(out short key);
 void SetHotkey(short key);
 void GetShowCmd(out int command);
 void SetShowCmd(int command);
 void GetIconLocation([Out, MarshalAs(UnmanagedType.LPWStr)] StringBuilder path,int capacity,out int index);
 void SetIconLocation([MarshalAs(UnmanagedType.LPWStr)] string path,int index);
 void SetRelativePath([MarshalAs(UnmanagedType.LPWStr)] string path,int reserved);
 void Resolve(IntPtr window,int flags);
 void SetPath([MarshalAs(UnmanagedType.LPWStr)] string path);
}
public static class IELTSStudioShortcut {
 public static string Target(string path){
  object link=new IELTSStudioShellLink();
  try {((IPersistFile)link).Load(path,0);var text=new StringBuilder(32768);((IELTSStudioShellLinkW)link).GetPath(text,text.Capacity,IntPtr.Zero,4);return text.ToString();}
  finally {Marshal.FinalReleaseComObject(link);}
 }
 public static void Save(string path,string target,string description){
  object link=new IELTSStudioShellLink();
  try {var shell=(IELTSStudioShellLinkW)link;shell.SetPath(target);shell.SetWorkingDirectory(Path.GetDirectoryName(target));shell.SetIconLocation(target,0);shell.SetDescription(description);((IPersistFile)link).Save(path,true);}
  finally {Marshal.FinalReleaseComObject(link);}
 }
}
'@
}
