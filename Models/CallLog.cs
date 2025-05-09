using System.ComponentModel.DataAnnotations;

namespace android_debugger.Models
{
    public class CallLog
    {
        [Key]
        public int Id { get; set; }
        public string Number { get; set; } = string.Empty;
        public string CallType { get; set; } = string.Empty; // Incoming, Outgoing, Missed
        public DateTime Date { get; set; }
        public int Duration { get; set; } // Duration in seconds
        public DateTime CreatedAt { get; set; } = DateTime.Now;
    }
}
