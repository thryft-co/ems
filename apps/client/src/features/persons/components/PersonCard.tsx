import React from "react";
import { Badge } from "@/shared/ui/badge";
import { Button } from "@/shared/ui/button";
import { Eye, Edit, Trash2, Mail, Phone, Building, User, Calendar, MapPin, Briefcase, Users, FileText } from "lucide-react";
import { PersonDetailResponse } from "@/features/persons/types/person";

interface PersonCardProps {
  person: PersonDetailResponse;
  onView?: (id: string) => void;
  onEdit?: (id: string) => void;
  onDelete?: (id: string) => void;
}

interface DetailItem { label: string; value: string; icon: React.ReactNode; }

const typeConfig: Record<string, { color: string; variant: "default" | "success" | "warning" | "secondary" }> = {
  internal: { color: "text-primary", variant: "default" },
  customer: { color: "text-success", variant: "success" },
  vendor: { color: "text-purple-600", variant: "secondary" },
  distributor: { color: "text-warning", variant: "warning" },
};

const PersonCard: React.FC<PersonCardProps> = ({ person, onView, onEdit, onDelete }) => {
  const typeInfo = typeConfig[person.person_type] || { color: "text-muted-foreground", variant: "secondary" as const };

  const getPersonTypeIcon = () => {
    switch (person.person_type) {
      case "internal": return <User className={`h-5 w-5 ${typeInfo.color}`} />;
      case "customer": return <Users className={`h-5 w-5 ${typeInfo.color}`} />;
      case "vendor": return <Building className={`h-5 w-5 ${typeInfo.color}`} />;
      case "distributor": return <MapPin className={`h-5 w-5 ${typeInfo.color}`} />;
      default: return <User className="h-5 w-5 text-muted-foreground" />;
    }
  };

  const getSpecificDetails = (): DetailItem[] => {
    const details: DetailItem[] = [];
    if (person.person_type === "internal" && person.internal) {
      details.push({ label: "Dept", value: person.internal.department || "N/A", icon: <Building className="h-3.5 w-3.5 text-muted-foreground/50" /> });
      details.push({ label: "Position", value: person.internal.position || "N/A", icon: <Briefcase className="h-3.5 w-3.5 text-muted-foreground/50" /> });
      if (person.internal.employee_id) details.push({ label: "ID", value: person.internal.employee_id, icon: <FileText className="h-3.5 w-3.5 text-muted-foreground/50" /> });
    } else if (person.person_type === "customer" && person.customer) {
      if (person.customer.company) details.push({ label: "Company", value: person.customer.company, icon: <Building className="h-3.5 w-3.5 text-muted-foreground/50" /> });
      if (person.customer.industry) details.push({ label: "Industry", value: person.customer.industry, icon: <Briefcase className="h-3.5 w-3.5 text-muted-foreground/50" /> });
    } else if (person.person_type === "vendor" && person.vendor) {
      details.push({ label: "Company", value: person.vendor.company || "N/A", icon: <Building className="h-3.5 w-3.5 text-muted-foreground/50" /> });
      if (person.vendor.service_type) details.push({ label: "Service", value: person.vendor.service_type, icon: <Briefcase className="h-3.5 w-3.5 text-muted-foreground/50" /> });
    } else if (person.person_type === "distributor" && person.distributor) {
      details.push({ label: "Company", value: person.distributor.company || "N/A", icon: <Building className="h-3.5 w-3.5 text-muted-foreground/50" /> });
      if (person.distributor.territory) details.push({ label: "Territory", value: person.distributor.territory, icon: <MapPin className="h-3.5 w-3.5 text-muted-foreground/50" /> });
    }
    return details;
  };

  const specificDetails = getSpecificDetails();

  return (
    <div className="bg-card rounded-2xl border-[0.5px] border-border/60 shadow-soft p-4 flex flex-col hover:shadow-soft-md transition-all duration-200 animate-fade-up">
      {/* Header */}
      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-full bg-secondary/60 flex items-center justify-center flex-shrink-0">
            {getPersonTypeIcon()}
          </div>
          <div className="min-w-0">
            <p className="text-[15px] font-semibold text-foreground leading-tight truncate">{person.name}</p>
            <p className="text-[12px] text-muted-foreground/60 truncate">{person.email}</p>
          </div>
        </div>
        <Badge variant={typeInfo.variant}>
          {person.person_type.charAt(0).toUpperCase() + person.person_type.slice(1)}
        </Badge>
      </div>

      {/* Contact info */}
      <div className="space-y-1.5 text-[13px] mb-3">
        <div className="flex items-center gap-2">
          <Mail className="h-3.5 w-3.5 text-muted-foreground/50 flex-shrink-0" />
          <span className="text-foreground/80 truncate">{person.email}</span>
        </div>
        {person.phone && (
          <div className="flex items-center gap-2">
            <Phone className="h-3.5 w-3.5 text-muted-foreground/50 flex-shrink-0" />
            <span className="text-foreground/80">{person.phone}</span>
          </div>
        )}
      </div>

      {/* Type-specific details */}
      {specificDetails.length > 0 && (
        <div className="grid grid-cols-2 gap-x-3 gap-y-1.5 text-[13px] mb-3 pt-3 border-t-[0.5px] border-border/40">
          {specificDetails.map((detail, i) => (
            <div key={i} className="flex items-center gap-2 overflow-hidden">
              {detail.icon}
              <span className="text-foreground/80 font-medium truncate">{detail.value}</span>
            </div>
          ))}
        </div>
      )}

      {/* Actions */}
      <div className="flex items-center gap-2 pt-3 border-t-[0.5px] border-border/40 mt-auto">
        {onView && <Button variant="ghost" size="sm" className="flex-1 h-9 text-[13px]" onClick={() => onView(person.id)}><Eye className="h-3.5 w-3.5 mr-1" />View</Button>}
        {onEdit && <Button variant="ghost" size="sm" className="flex-1 h-9 text-[13px]" onClick={() => onEdit(person.id)}><Edit className="h-3.5 w-3.5 mr-1" />Edit</Button>}
        {onDelete && <Button variant="ghost" size="sm" className="h-9 text-[13px] text-destructive hover:bg-destructive/8" onClick={() => onDelete(person.id)}><Trash2 className="h-3.5 w-3.5" /></Button>}
      </div>
    </div>
  );
};

export default PersonCard;
