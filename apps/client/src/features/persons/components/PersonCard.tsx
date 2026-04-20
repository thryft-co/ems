import React from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/shared/ui/card";
import { Badge } from "@/shared/ui/badge";
import { Button } from "@/shared/ui/button";
import {
  Eye,
  Edit,
  Trash2,
  Mail,
  Phone,
  Building,
  User,
  Calendar,
  MapPin,
  Briefcase,
  Users,
  FileText,
} from "lucide-react";
import { PersonDetailResponse } from "@/features/persons/types/person";

interface PersonCardProps {
  person: PersonDetailResponse;
  onView?: (id: string) => void;
  onEdit?: (id: string) => void;
  onDelete?: (id: string) => void;
}

interface DetailItem {
  label: string;
  value: string;
  icon: React.ReactNode;
}

const PersonCard: React.FC<PersonCardProps> = ({
  person,
  onView,
  onEdit,
  onDelete,
}) => {
  const getPersonTypeColor = (type: string): string => {
    const typeColors: { [key: string]: string } = {
      internal: "bg-blue-100 text-blue-800 border-blue-200",
      customer: "bg-green-100 text-green-800 border-green-200",
      vendor: "bg-purple-100 text-purple-800 border-purple-200",
      distributor: "bg-orange-100 text-orange-800 border-orange-200",
    };
    return typeColors[type] || "bg-gray-100 text-gray-800 border-gray-200";
  };

  const getSpecificDetails = (): DetailItem[] => {
    const details: DetailItem[] = [];

    if (person.person_type === "internal" && person.internal) {
      details.push(
        {
          label: "Department",
          value: person.internal.department || "N/A",
          icon: <Building className="h-4 w-4" />,
        },
        {
          label: "Position",
          value: person.internal.position || "N/A",
          icon: <Briefcase className="h-4 w-4" />,
        },
        {
          label: "Employee ID",
          value: person.internal.employee_id || "N/A",
          icon: <FileText className="h-4 w-4" />,
        },
      );
      if (person.internal.hire_date) {
        details.push({
          label: "Hire Date",
          value: new Date(person.internal.hire_date).toLocaleDateString(),
          icon: <Calendar className="h-4 w-4" />,
        });
      }
    } else if (person.person_type === "customer" && person.customer) {
      if (person.customer.company) {
        details.push({
          label: "Company",
          value: person.customer.company,
          icon: <Building className="h-4 w-4" />,
        });
      }
      if (person.customer.industry) {
        details.push({
          label: "Industry",
          value: person.customer.industry,
          icon: <Briefcase className="h-4 w-4" />,
        });
      }
      if (person.customer.customer_since) {
        details.push({
          label: "Customer Since",
          value: new Date(person.customer.customer_since).toLocaleDateString(),
          icon: <Calendar className="h-4 w-4" />,
        });
      }
    } else if (person.person_type === "vendor" && person.vendor) {
      details.push({
        label: "Company",
        value: person.vendor.company || "N/A",
        icon: <Building className="h-4 w-4" />,
      });
      if (person.vendor.service_type) {
        details.push({
          label: "Service Type",
          value: person.vendor.service_type,
          icon: <Briefcase className="h-4 w-4" />,
        });
      }
      if (person.vendor.contract_start) {
        details.push({
          label: "Contract Start",
          value: new Date(person.vendor.contract_start).toLocaleDateString(),
          icon: <Calendar className="h-4 w-4" />,
        });
      }
    } else if (person.person_type === "distributor" && person.distributor) {
      details.push({
        label: "Company",
        value: person.distributor.company || "N/A",
        icon: <Building className="h-4 w-4" />,
      });
      if (person.distributor.territory) {
        details.push({
          label: "Territory",
          value: person.distributor.territory,
          icon: <MapPin className="h-4 w-4" />,
        });
      }
      if (person.distributor.distribution_tier) {
        details.push({
          label: "Tier",
          value: person.distributor.distribution_tier,
          icon: <Users className="h-4 w-4" />,
        });
      }
    }

    return details;
  };

  const getPersonTypeIcon = () => {
    switch (person.person_type) {
      case "internal":
        return <User className="h-5 w-5 text-blue-600" />;
      case "customer":
        return <Users className="h-5 w-5 text-green-600" />;
      case "vendor":
        return <Building className="h-5 w-5 text-purple-600" />;
      case "distributor":
        return <MapPin className="h-5 w-5 text-orange-600" />;
      default:
        return <User className="h-5 w-5 text-gray-600" />;
    }
  };

  const getPersonTypeLabel = () => {
    return (
      person.person_type.charAt(0).toUpperCase() + person.person_type.slice(1)
    );
  };

  const specificDetails = getSpecificDetails();

  return (
    <Card className="hover:shadow-md transition-shadow duration-200">
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between">
          <div className="flex items-center space-x-2">
            {getPersonTypeIcon()}
            <div>
              <CardTitle className="text-lg font-semibold text-gray-900">
                {person.name}
              </CardTitle>
              <CardDescription className="text-sm text-gray-600">
                {person.email}
              </CardDescription>
            </div>
          </div>
          <Badge
            className={`${getPersonTypeColor(person.person_type)} text-xs font-medium`}
          >
            {getPersonTypeLabel()}
          </Badge>
        </div>
      </CardHeader>

      <CardContent className="space-y-4">
        {/* Contact Information */}
        <div className="space-y-2">
          <div className="flex items-center space-x-2 text-sm">
            <Mail className="h-4 w-4 text-gray-500" />
            <span className="text-gray-600">Email:</span>
            <span className="font-medium">{person.email}</span>
          </div>
          {person.phone && (
            <div className="flex items-center space-x-2 text-sm">
              <Phone className="h-4 w-4 text-gray-500" />
              <span className="text-gray-600">Phone:</span>
              <span className="font-medium">{person.phone}</span>
            </div>
          )}
        </div>

        {/* Type-specific details */}
        {specificDetails.length > 0 && (
          <div className="space-y-2 pt-2 border-t">
            {specificDetails.map((detail, index) => (
              <div key={index} className="flex items-center space-x-2 text-sm">
                {detail.icon}
                <span className="text-gray-600">{detail.label}:</span>
                <span className="font-medium">{detail.value}</span>
              </div>
            ))}
          </div>
        )}

        {/* Action buttons */}
        <div className="flex space-x-2 pt-3 border-t">
          {onView && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => onView(person.id)}
              className="flex-1"
            >
              <Eye className="h-4 w-4 mr-1" />
              View
            </Button>
          )}
          {onEdit && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => onEdit(person.id)}
              className="flex-1"
            >
              <Edit className="h-4 w-4 mr-1" />
              Edit
            </Button>
          )}
          {onDelete && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => onDelete(person.id)}
              className="text-red-600 hover:text-red-700 hover:bg-red-50"
            >
              <Trash2 className="h-4 w-4" />
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  );
};

export default PersonCard;
