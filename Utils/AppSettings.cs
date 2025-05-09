using android_debugger.Data;
using Microsoft.EntityFrameworkCore;
using System.Text;

namespace android_debugger.Utils
{
    public static class AppSettings
    {
        // Paths
        public static string AppDataPath => Path.Combine(
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            "AndroidDebugger");

        public static string DbFilePath => Path.Combine(AppDataPath, "AndroidExtractor.mdf");
        
        // ADB Path - can be customized by the user
        public static string AdbPath { get; set; } = "adb";

        // Connection string for SQL Server LocalDB
        public static string GetConnectionString()
        {
            // Ensure app data directory exists
            if (!Directory.Exists(AppDataPath))
            {
                Directory.CreateDirectory(AppDataPath);
            }

            return $"Server=(LocalDB)\\MSSQLLocalDB;AttachDbFilename={DbFilePath};Integrated Security=True;Connect Timeout=30;";
        }

        // Export data to CSV
        public static bool ExportToCSV<T>(List<T> data, string fileName, string[] headers)
        {
            try
            {
                string filePath = Path.Combine(
                    Environment.GetFolderPath(Environment.SpecialFolder.MyDocuments),
                    "AndroidDebugger",
                    fileName);

                // Ensure directory exists
                Directory.CreateDirectory(Path.GetDirectoryName(filePath)!);

                // Create CSV file
                using var writer = new StreamWriter(filePath, false, Encoding.UTF8);

                // Write headers
                writer.WriteLine(string.Join(",", headers));

                // Write data rows
                foreach (var item in data)
                {
                    var values = item!.GetType().GetProperties()
                        .Where(p => headers.Any(h => h.Equals(p.Name, StringComparison.OrdinalIgnoreCase)))
                        .Select(p => {
                            var value = p.GetValue(item)?.ToString() ?? "";
                            // Escape commas and quotes
                            if (value.Contains(",") || value.Contains("\"") || value.Contains("\r") || value.Contains("\n"))
                            {
                                return $"\"{value.Replace("\"", "\"\"")}\"";
                            }
                            return value;
                        });

                    writer.WriteLine(string.Join(",", values));
                }

                return true;
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Error exporting data: {ex.Message}", "Export Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
                return false;
            }
        }

        // Initialize database
        public static async Task InitializeDatabaseAsync()
        {
            try
            {
                // Create database context options
                var optionsBuilder = new DbContextOptionsBuilder<AndroidDbContext>();
                optionsBuilder.UseSqlServer(GetConnectionString());

                // Create and migrate database
                using var dbContext = new AndroidDbContext(optionsBuilder.Options);
                await dbContext.Database.EnsureCreatedAsync();
            }
            catch (Exception ex)
            {
                MessageBox.Show($"Error initializing database: {ex.Message}", "Database Error", MessageBoxButtons.OK, MessageBoxIcon.Error);
            }
        }
    }
}
