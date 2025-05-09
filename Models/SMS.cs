using System.ComponentModel.DataAnnotations;

namespace android_debugger.Models
{
    public class SMS
    {
        [Key]
        public int Id { get; set; }
        public string Address { get; set; } = string.Empty;
        public string Body { get; set; } = string.Empty;
        public DateTime Date { get; set; }
        public string Type { get; set; } = string.Empty; // Inbox, Outbox, etc.
        public DateTime CreatedAt { get; set; } = DateTime.Now;
    }
}