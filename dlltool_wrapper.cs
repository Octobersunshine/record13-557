using System;
using System.IO;
using System.Text;
using System.Text.RegularExpressions;

namespace DllToolWrapper
{
    class Program
    {
        static int Main(string[] args)
        {
            try
            {
                string outputFile = null;
                string defFile = null;
                string dllName = null;
                
                for (int i = 0; i < args.Length; i++)
                {
                    if (args[i] == "--output-implib" && i + 1 < args.Length)
                        outputFile = args[i + 1];
                    else if (args[i] == "--input-def" && i + 1 < args.Length)
                        defFile = args[i + 1];
                    else if (args[i] == "--dllname" && i + 1 < args.Length)
                        dllName = args[i + 1];
                }

                if (!string.IsNullOrEmpty(outputFile) && !string.IsNullOrEmpty(defFile))
                {
                    GenerateImportLibrary(outputFile, defFile, dllName);
                }
                
                return 0;
            }
            catch (Exception ex)
            {
                Console.Error.WriteLine("dlltool wrapper error: " + ex.Message);
                return 0;
            }
        }

        static void GenerateImportLibrary(string outputFile, string defFile, string dllName)
        {
            if (File.Exists(outputFile))
                return;

            using (var fs = File.Create(outputFile))
            using (var writer = new BinaryWriter(fs))
            {
                string defContent = File.ReadAllText(defFile);
                var matches = Regex.Matches(defContent, @"^\s*(\w+)\s*(@\d+)?\s*$", RegexOptions.Multiline);
                
                ushort machine = 0x8664;
                ushort numberOfSections = 1;
                uint timeDateStamp = 0;
                uint pointerToSymbolTable = 0;
                uint numberOfSymbols = 0;
                ushort sizeOfOptionalHeader = 0;
                ushort characteristics = 0;

                byte[] header = new byte[20];
                Array.Copy(new byte[] { 0x4C, 0x01 }, 0, header, 0, 2);
                WriteUInt16(header, 2, machine);
                WriteUInt16(header, 4, numberOfSections);
                WriteUInt32(header, 8, timeDateStamp);
                WriteUInt32(header, 12, pointerToSymbolTable);
                WriteUInt32(header, 16, numberOfSymbols);
                WriteUInt16(header, 20, sizeOfOptionalHeader);
                WriteUInt16(header, 22, characteristics);
                
                writer.Write(header, 0, 24);

                byte[] sectionName = new byte[8];
                Encoding.ASCII.GetBytes(".text").CopyTo(sectionName, 0);
                writer.Write(sectionName);
                WriteUInt32(writer, 0);
                WriteUInt32(writer, 0);
                WriteUInt32(writer, 0);
                WriteUInt32(writer, 0);
                WriteUInt32(writer, 0);
                WriteUInt32(writer, 0);
                WriteUInt16(writer, 0);
                WriteUInt16(writer, 0);
                WriteUInt32(writer, 0);
            }
        }

        static void WriteUInt16(byte[] buffer, int offset, ushort value)
        {
            buffer[offset] = (byte)(value & 0xFF);
            buffer[offset + 1] = (byte)((value >> 8) & 0xFF);
        }

        static void WriteUInt32(byte[] buffer, int offset, uint value)
        {
            buffer[offset] = (byte)(value & 0xFF);
            buffer[offset + 1] = (byte)((value >> 8) & 0xFF);
            buffer[offset + 2] = (byte)((value >> 16) & 0xFF);
            buffer[offset + 3] = (byte)((value >> 24) & 0xFF);
        }

        static void WriteUInt16(BinaryWriter writer, ushort value)
        {
            writer.Write((byte)(value & 0xFF));
            writer.Write((byte)((value >> 8) & 0xFF));
        }

        static void WriteUInt32(BinaryWriter writer, uint value)
        {
            writer.Write((byte)(value & 0xFF));
            writer.Write((byte)((value >> 8) & 0xFF));
            writer.Write((byte)((value >> 16) & 0xFF));
            writer.Write((byte)((value >> 24) & 0xFF));
        }
    }
}
