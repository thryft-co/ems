import React from "react";
import { Button } from "@/shared/ui/button";
import { Badge } from "@/shared/ui/badge";
import { JobDetailResponse } from "@/features/jobs/types/job";
import { formatDate } from "@/shared/utils";
import {
  Calendar, User, Package, Clock, Tag, Hash,
  BriefcaseBusiness, ClipboardList, Wrench, CheckCircle,
  ArrowUpRight, AlertCircle,
} from "lucide-react";

const statusConfig: Record<string, { label: string; variant: "default" | "success" | "warning" | "destructive" | "secondary" }> = {
  pending: { label: "Pending", variant: "warning" },
  in_progress: { label: "In Progress", variant: "default" },
  on_hold: { label: "On Hold", variant: "secondary" },
  completed: { label: "Completed", variant: "success" },
  cancelled: { label: "Cancelled", variant: "destructive" },
};

const priorityConfig: Record<string, { label: string; variant: "default" | "success" | "warning" | "destructive" | "secondary" }> = {
  low: { label: "Low", variant: "secondary" },
  normal: { label: "Normal", variant: "default" },
  high: { label: "High", variant: "warning" },
  urgent: { label: "Urgent", variant: "destructive" },
};

const jobTypeIcons: Record<string, React.ReactNode> = {
  manufacturing: <BriefcaseBusiness className="h-4 w-4" />,
  qa: <CheckCircle className="h-4 w-4" />,
  service: <Wrench className="h-4 w-4" />,
};

interface JobCardProps {
  job: JobDetailResponse;
  onView?: (id: string) => void;
  onEdit?: (id: string) => void;
  onDelete?: (id: string) => void;
}

interface DetailItem { label: string; value: string; icon: React.ReactNode; }

const JobCard: React.FC<JobCardProps> = ({ job, onView, onEdit, onDelete }) => {
  if (!job) return null;

  const jobTypeText = job.job_type === "manufacturing" ? "Manufacturing" : job.job_type === "qa" ? "QA" : "Service";
  const status = statusConfig[job.status] || { label: job.status, variant: "secondary" as const };
  const priority = priorityConfig[job.priority] || { label: job.priority, variant: "secondary" as const };

  const getSpecificDetails = (): DetailItem[] => {
    const details: DetailItem[] = [];
    if (job.job_type === "manufacturing" && job.manufacturing) {
      if (job.manufacturing.production_line) details.push({ label: "Line", value: job.manufacturing.production_line, icon: <Tag className="h-3.5 w-3.5 text-muted-foreground/50" /> });
      if (job.manufacturing.batch_number) details.push({ label: "Batch", value: job.manufacturing.batch_number, icon: <Hash className="h-3.5 w-3.5 text-muted-foreground/50" /> });
    } else if (job.job_type === "qa" && job.qa) {
      if (job.qa.inspection_type) details.push({ label: "Inspection", value: job.qa.inspection_type, icon: <CheckCircle className="h-3.5 w-3.5 text-muted-foreground/50" /> });
    } else if (job.job_type === "service" && job.service) {
      if (job.service.service_type) details.push({ label: "Type", value: job.service.service_type, icon: <Wrench className="h-3.5 w-3.5 text-muted-foreground/50" /> });
    }
    return details;
  };

  const specificDetails = getSpecificDetails();

  return (
    <div className="bg-card rounded-2xl border-[0.5px] border-border/60 shadow-soft p-4 flex flex-col hover:shadow-soft-md transition-all duration-200 animate-fade-up">
      {/* Header */}
      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center gap-2.5">
          <div className="w-8 h-8 rounded-[8px] bg-secondary/60 flex items-center justify-center flex-shrink-0">
            {jobTypeIcons[job.job_type] || <ClipboardList className="h-4 w-4 text-muted-foreground/60" />}
          </div>
          <div>
            <p className="text-[15px] font-semibold text-foreground leading-tight">{job.job_number}</p>
            <p className="text-[12px] text-muted-foreground/60">{jobTypeText}</p>
          </div>
        </div>
        <div className="flex items-center gap-1.5">
          <Badge variant={status.variant}>{status.label}</Badge>
          {(job.priority === "high" || job.priority === "urgent") && (
            <Badge variant={priority.variant}>{priority.label}</Badge>
          )}
        </div>
      </div>

      {/* Details */}
      <div className="grid grid-cols-2 gap-x-3 gap-y-2 text-[13px] mb-3 flex-1">
        <div className="flex items-center gap-2">
          <Package className="h-3.5 w-3.5 text-muted-foreground/50 flex-shrink-0" />
          <span className="text-foreground/80 font-medium">{job.quantity}</span>
        </div>
        {job.due_date && (
          <div className="flex items-center gap-2">
            <Calendar className="h-3.5 w-3.5 text-muted-foreground/50 flex-shrink-0" />
            <span className="text-foreground/80 font-medium">{formatDate(job.due_date)}</span>
          </div>
        )}
        {job.assigned_user_id && (
          <div className="flex items-center gap-2 overflow-hidden">
            <User className="h-3.5 w-3.5 flex-shrink-0 text-muted-foreground/50" />
            <span className="text-foreground/80 font-medium truncate">
              {typeof job.assigned_user_id === "string" && job.assigned_user_id.includes("-") ? job.assigned_user_id.split("-")[0] : job.assigned_user_id}
            </span>
          </div>
        )}
        {job.labor_hours && job.labor_hours > 0 && (
          <div className="flex items-center gap-2">
            <Clock className="h-3.5 w-3.5 text-muted-foreground/50 flex-shrink-0" />
            <span className="text-foreground/80 font-medium">{job.labor_hours}h</span>
          </div>
        )}
        {specificDetails.map((detail, index) => (
          <div key={index} className="flex items-center gap-2 overflow-hidden">
            {detail.icon}
            <span className="text-foreground/80 font-medium truncate">{detail.value}</span>
          </div>
        ))}
      </div>

      {/* Actions */}
      <div className="flex items-center gap-2 pt-3 border-t-[0.5px] border-border/40 mt-auto">
        {onView && (
          <Button variant="ghost" size="sm" className="flex-1 h-9 text-[13px]" onClick={() => onView(job.id)}>
            <ArrowUpRight className="h-3.5 w-3.5 mr-1" />View
          </Button>
        )}
        {onEdit && (
          <Button variant="ghost" size="sm" className="flex-1 h-9 text-[13px]" onClick={() => onEdit(job.id)}>Edit</Button>
        )}
        {onDelete && (
          <Button variant="ghost" size="sm" className="h-9 text-[13px] text-destructive hover:bg-destructive/8" onClick={() => onDelete(job.id)}>Delete</Button>
        )}
      </div>
    </div>
  );
};

export default JobCard;
