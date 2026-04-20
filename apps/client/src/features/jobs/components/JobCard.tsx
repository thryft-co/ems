import React from "react";
import { Button } from "@/shared/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from "@/shared/ui/card";
import { JobDetailResponse } from "@/features/jobs/types/job";
import { formatDate } from "@/shared/utils";
import {
  Calendar,
  User,
  Package,
  Clock,
  Tag,
  Hash,
  BriefcaseBusiness,
  ClipboardList,
  Wrench,
  CheckCircle,
  ArrowUpRight,
  AlertCircle,
} from "lucide-react";

// Define status and priority color mappings
const statusColors: Record<string, string> = {
  pending: "bg-yellow-500",
  in_progress: "bg-blue-500",
  on_hold: "bg-orange-500",
  completed: "bg-green-500",
  cancelled: "bg-red-500",
};

const priorityColors: Record<string, string> = {
  low: "bg-slate-400",
  normal: "bg-blue-400",
  high: "bg-amber-500",
  urgent: "bg-red-500",
};

// Priority numbers
const priorityNumbers: Record<string, number> = {
  low: 1,
  normal: 2,
  high: 3,
  urgent: 4,
};

// Job type icons mapping
const jobTypeIcons: Record<string, React.ReactNode> = {
  manufacturing: <BriefcaseBusiness className="h-4 w-4" />,
  qa: <CheckCircle className="h-4 w-4" />,
  service: <Wrench className="h-4 w-4" />,
};

// Define props interface for the component
interface JobCardProps {
  job: JobDetailResponse;
  onView?: (id: string) => void;
  onEdit?: (id: string) => void;
  onDelete?: (id: string) => void;
}

// Type for specific job details
interface DetailItem {
  label: string;
  value: string;
  icon: React.ReactNode;
}

const JobCard: React.FC<JobCardProps> = ({ job, onView, onEdit, onDelete }) => {
  if (!job) return null;

  // Format job type display text
  const jobTypeText =
    job.job_type === "manufacturing"
      ? "Manufacturing"
      : job.job_type === "qa"
        ? "QA"
        : "Service";

  // Get specific details based on job type
  const getSpecificDetails = (): DetailItem[] => {
    const details: DetailItem[] = [];

    if (job.job_type === "manufacturing" && job.manufacturing) {
      if (job.manufacturing.production_line) {
        details.push({
          label: "Production Line",
          value: job.manufacturing.production_line,
          icon: <Tag className="h-3 w-3 flex-shrink-0 text-gray-500" />,
        });
      }
      if (job.manufacturing.batch_number) {
        details.push({
          label: "Batch Number",
          value: job.manufacturing.batch_number,
          icon: <Hash className="h-3 w-3 flex-shrink-0 text-gray-500" />,
        });
      }
    } else if (job.job_type === "qa" && job.qa) {
      if (job.qa.inspection_type) {
        details.push({
          label: "Inspection Type",
          value: job.qa.inspection_type,
          icon: <CheckCircle className="h-3 w-3 flex-shrink-0 text-gray-500" />,
        });
      }
    } else if (job.job_type === "service" && job.service) {
      if (job.service.service_type) {
        details.push({
          label: "Service Type",
          value: job.service.service_type,
          icon: <Wrench className="h-3 w-3 flex-shrink-0 text-gray-500" />,
        });
      }
    }
    return details;
  };

  const specificDetails = getSpecificDetails();
  const priorityNumber = priorityNumbers[job.priority] || 2;

  return (
    <Card className="w-full h-[200px] mb-3 hover:shadow-md transition-shadow flex flex-col">
      <CardHeader className="p-3 pb-2 flex-shrink-0">
        <div className="flex justify-between items-center">
          <div className="flex items-center gap-2">
            <div className="bg-slate-100 p-1 rounded">
              {jobTypeIcons[job.job_type] || (
                <ClipboardList className="h-4 w-4" />
              )}
            </div>
            <div>
              <CardTitle className="text-base">{job.job_number}</CardTitle>
              <CardDescription className="text-xs mt-0">
                {jobTypeText}
              </CardDescription>
            </div>
          </div>

          <div className="flex gap-2 items-center">
            {/* Status indicator dot */}
            <div
              className={`h-3 w-3 rounded-full ${statusColors[job.status] || "bg-gray-400"}`}
              title={`Status: ${job.status?.toUpperCase() || "UNKNOWN"}`}
            />

            {/* Priority indicator with number */}
            <div
              className={`h-5 w-5 rounded-full ${priorityColors[job.priority] || "bg-gray-400"} flex items-center justify-center`}
              title={`Priority: ${job.priority?.toUpperCase() || "NORMAL"}`}
            >
              <span className="text-white text-xs font-bold">
                {priorityNumber}
              </span>
            </div>

            {job.status === "on_hold" && (
              <div title="On Hold">
                <AlertCircle className="h-4 w-4 text-orange-500" />
              </div>
            )}
            {job.priority === "urgent" && (
              <div title="Urgent">
                <AlertCircle className="h-4 w-4 text-red-500" />
              </div>
            )}
          </div>
        </div>
      </CardHeader>

      <CardContent className="p-3 pt-1 overflow-y-auto flex-grow">
        <div className="grid grid-cols-2 gap-x-3 gap-y-2 text-sm">
          <div className="flex items-center gap-2" title="Quantity">
            <Package className="h-4 w-4 text-gray-500" />
            <span className="font-medium">{job.quantity}</span>
          </div>

          {job.due_date && (
            <div className="flex items-center gap-2" title="Due Date">
              <Calendar className="h-4 w-4 text-gray-500" />
              <span className="font-medium">{formatDate(job.due_date)}</span>
            </div>
          )}

          {job.assigned_user_id && (
            <div
              className="flex items-center gap-2 overflow-hidden"
              title={`Assigned to: ${job.assigned_user_id}`}
            >
              <User className="h-4 w-4 flex-shrink-0 text-gray-500" />
              <span className="font-medium truncate">
                {typeof job.assigned_user_id === "string" &&
                job.assigned_user_id.includes("-")
                  ? job.assigned_user_id.split("-")[0]
                  : job.assigned_user_id}
              </span>
            </div>
          )}

          {job.labor_hours && job.labor_hours > 0 && (
            <div className="flex items-center gap-2" title="Labor Hours">
              <Clock className="h-4 w-4 text-gray-500" />
              <span className="font-medium">{job.labor_hours}</span>
            </div>
          )}

          {/* Display specific details for each job type */}
          {specificDetails.map((detail, index) => (
            <div
              key={index}
              className="flex items-center gap-2 overflow-hidden"
              title={`${detail.label}: ${detail.value}`}
            >
              {detail.icon}
              <span className="font-medium truncate">{detail.value}</span>
            </div>
          ))}

          {/* Item ID shortened */}
          <div
            className="flex items-center gap-2 col-span-2 overflow-hidden"
            title={`Item ID: ${job.item_id}`}
          >
            <Hash className="h-4 w-4 flex-shrink-0 text-gray-500" />
            <span className="font-medium truncate">
              {typeof job.item_id === "string" && job.item_id.includes("-")
                ? `${job.item_id.split("-")[0]}...`
                : job.item_id}
            </span>
          </div>
        </div>
      </CardContent>

      <CardFooter className="p-2 flex justify-end gap-1 border-t mt-auto flex-shrink-0">
        {onView && (
          <Button
            variant="ghost"
            size="sm"
            className="h-8"
            onClick={() => {
              console.log(`JobCard: View button clicked for job ID: ${job.id}`);
              onView(job.id);
            }}
          >
            <ArrowUpRight className="h-4 w-4 mr-1" />
            View
          </Button>
        )}
        {onEdit && (
          <Button
            variant="outline"
            size="sm"
            className="h-8"
            onClick={() => {
              console.log(`JobCard: Edit button clicked for job ID: ${job.id}`);
              onEdit(job.id);
            }}
          >
            Edit
          </Button>
        )}
        {onDelete && (
          <Button
            variant="destructive"
            size="sm"
            className="h-8"
            onClick={() => {
              console.log(
                `JobCard: Delete button clicked for job ID: ${job.id}`,
              );
              onDelete(job.id);
            }}
          >
            Delete
          </Button>
        )}
      </CardFooter>
    </Card>
  );
};

export default JobCard;
