using android_debugger.Data;
using android_debugger.Services;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.DependencyInjection;

namespace android_debugger
{
    internal static class Program
    {
        /// <summary>
        ///  The main entry point for the application.
        /// </summary>
        [STAThread]
        static void Main()
        {
            // To customize application configuration such as set high DPI settings or default font,
            // see https://aka.ms/applicationconfiguration.
            ApplicationConfiguration.Initialize();

            try
            {
                // Ensure ADB is accessible
                CheckAndInstallAdb();

                Application.Run(new MainForm());
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Error starting application: {ex.Message}", "Application Error", 
                    MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }

        /// <summary>
        /// Check if ADB is installed and accessible, if not provide instructions
        /// </summary>
        private static void CheckAndInstallAdb()
        {
            try
            {
                // Try to execute a simple ADB command
                var process = new System.Diagnostics.Process
                {
                    StartInfo = new System.Diagnostics.ProcessStartInfo
                    {
                        FileName = "adb",
                        Arguments = "version",
                        RedirectStandardOutput = true,
                        RedirectStandardError = true,
                        UseShellExecute = false,
                        CreateNoWindow = true
                    }
                };

                process.Start();
                string output = process.StandardOutput.ReadToEnd();
                process.WaitForExit();

                if (string.IsNullOrEmpty(output) || !output.Contains("Android Debug Bridge"))
                {
                    ShowAdbInstructions();
                }
            }
            catch
            {
                ShowAdbInstructions();
            }
        }

        /// <summary>
        /// Shows instructions for installing ADB
        /// </summary>
        private static void ShowAdbInstructions()
        {
            MessageBox.Show(
                "Android Debug Bridge (ADB) is not installed or not properly configured.\n\n" +
                "To use this application, please install Android SDK Platform-Tools which includes ADB.\n\n" +
                "1. Visit https://developer.android.com/studio/releases/platform-tools\n" +
                "2. Download the Platform Tools package for your OS\n" +
                "3. Extract the ZIP file to a location on your computer\n" +
                "4. Add the extracted folder to your system PATH\n\n" +
                "The application will continue to run, but device extraction features will not work until ADB is properly configured.",
                "ADB Not Found", MessageBoxButtons.OK, MessageBoxIcon.Warning);
        }
    }
}