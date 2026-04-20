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
  Calendar,
  DollarSign,
  User,
  Building,
  MapPin,
  Truck,
  CreditCard,
  Package,
  Users,
  FileText,
  CheckCircle,
} from "lucide-react";
import { OrderDetailResponse } from "@/features/orders/types/order";

interface OrderCardProps {
  order: OrderDetailResponse;
  onView?: (id: string) => void;
  onEdit?: (id: string) => void;
  onDelete?: (id: string) => void;
}

interface DetailItem {
  label: string;
  value: string;
  icon: React.ReactNode;
}

const OrderCard: React.FC<OrderCardProps> = ({
  order,
  onView,
  onEdit,
  onDelete,
}) => {
  const getStatusColor = (status: string): string => {
    const statusColors: { [key: string]: string } = {
      draft: "bg-gray-100 text-gray-800",
      submitted: "bg-blue-100 text-blue-800",
      approved: "bg-green-100 text-green-800",
      fulfilled: "bg-purple-100 text-purple-800",
      partially_fulfilled: "bg-yellow-100 text-yellow-800",
      cancelled: "bg-red-100 text-red-800",
      paid: "bg-emerald-100 text-emerald-800",
    };
    return statusColors[status] || "bg-gray-100 text-gray-800";
  };

  const getSpecificDetails = (): DetailItem[] => {
    switch (order.order_type) {
      case "customer_order":
        return [
          {
            label: "Customer Reference",
            value: order.customer_order?.customer_reference || "N/A",
            icon: <FileText className="h-4 w-4" />,
          },
          {
            label: "Shipping Address",
            value: order.customer_order?.shipping_address || "N/A",
            icon: <MapPin className="h-4 w-4" />,
          },
          {
            label: "Payment Method",
            value: order.customer_order?.payment_method || "N/A",
            icon: <CreditCard className="h-4 w-4" />,
          },
          {
            label: "Shipping Method",
            value: order.customer_order?.shipping_method || "N/A",
            icon: <Truck className="h-4 w-4" />,
          },
        ];

      case "purchase_order":
        return [
          {
            label: "Vendor Reference",
            value: order.purchase_order?.vendor_reference || "N/A",
            icon: <FileText className="h-4 w-4" />,
          },
          {
            label: "Payment Terms",
            value: order.purchase_order?.payment_terms || "N/A",
            icon: <CreditCard className="h-4 w-4" />,
          },
          {
            label: "Shipping Terms",
            value: order.purchase_order?.shipping_terms || "N/A",
            icon: <Truck className="h-4 w-4" />,
          },
          {
            label: "Expected Delivery",
            value: order.purchase_order?.expected_delivery_date
              ? new Date(
                  order.purchase_order.expected_delivery_date,
                ).toLocaleDateString()
              : "N/A",
            icon: <Calendar className="h-4 w-4" />,
          },
        ];

      case "distributor_order":
        return [
          {
            label: "Territory",
            value: order.distributor_order?.territory || "N/A",
            icon: <MapPin className="h-4 w-4" />,
          },
          {
            label: "Commission Rate",
            value: order.distributor_order?.commission_rate
              ? `${order.distributor_order.commission_rate}%`
              : "N/A",
            icon: <DollarSign className="h-4 w-4" />,
          },
          {
            label: "Agreement Reference",
            value: order.distributor_order?.agreement_reference || "N/A",
            icon: <FileText className="h-4 w-4" />,
          },
          {
            label: "Target Resale Amount",
            value: order.distributor_order?.target_resale_amount
              ? `$${order.distributor_order.target_resale_amount.toLocaleString()}`
              : "N/A",
            icon: <DollarSign className="h-4 w-4" />,
          },
        ];

      default:
        return [];
    }
  };

  const getOrderTypeIcon = () => {
    switch (order.order_type) {
      case "customer_order":
        return <User className="h-5 w-5" />;
      case "purchase_order":
        return <Package className="h-5 w-5" />;
      case "distributor_order":
        return <Users className="h-5 w-5" />;
      default:
        return <FileText className="h-5 w-5" />;
    }
  };

  const getOrderTypeLabel = () => {
    switch (order.order_type) {
      case "customer_order":
        return "Customer Order";
      case "purchase_order":
        return "Purchase Order";
      case "distributor_order":
        return "Distributor Order";
      default:
        return order.order_type;
    }
  };

  const specificDetails = getSpecificDetails();
  const orderDate = order.order_date
    ? new Date(order.order_date).toLocaleDateString()
    : "N/A";

  return (
    <Card className="w-full hover:shadow-lg transition-shadow duration-200">
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            {getOrderTypeIcon()}
            <CardTitle className="text-lg font-semibold">
              {order.order_number}
            </CardTitle>
          </div>
          <Badge
            className={`${getStatusColor(order.status)} text-xs font-medium`}
          >
            {order.status.replace("_", " ").toUpperCase()}
          </Badge>
        </div>
        <CardDescription className="text-sm text-gray-600">
          {getOrderTypeLabel()}
        </CardDescription>
      </CardHeader>

      <CardContent className="pt-0">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3 mb-4">
          <div className="flex items-center space-x-2 text-sm">
            <Calendar className="h-4 w-4 text-gray-500" />
            <span className="text-gray-600">Date:</span>
            <span className="font-medium">{orderDate}</span>
          </div>
          <div className="flex items-center space-x-2 text-sm">
            <DollarSign className="h-4 w-4 text-gray-500" />
            <span className="text-gray-600">Total:</span>
            <span className="font-medium">
              ${order.total_amount.toLocaleString()}
            </span>
          </div>
        </div>

        {/* Order-specific details */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-2 mb-4">
          {specificDetails.map((detail, index) => (
            <div key={index} className="flex items-center space-x-2 text-sm">
              <span className="text-gray-500">{detail.icon}</span>
              <span className="text-gray-600 truncate">{detail.label}:</span>
              <span className="font-medium truncate" title={detail.value}>
                {detail.value}
              </span>
            </div>
          ))}
        </div>

        {/* Order items count */}
        {order.items && order.items.length > 0 && (
          <div className="flex items-center space-x-2 text-sm mb-4">
            <Package className="h-4 w-4 text-gray-500" />
            <span className="text-gray-600">Items:</span>
            <span className="font-medium">{order.items.length}</span>
          </div>
        )}

        {/* Action buttons */}
        <div className="flex space-x-2 pt-3 border-t border-gray-100">
          {onView && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => onView(order.id)}
              className="flex items-center space-x-1"
            >
              <Eye className="h-4 w-4" />
              <span>View</span>
            </Button>
          )}
          {onEdit && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => onEdit(order.id)}
              className="flex items-center space-x-1"
            >
              <Edit className="h-4 w-4" />
              <span>Edit</span>
            </Button>
          )}
          {onDelete && (
            <Button
              variant="outline"
              size="sm"
              onClick={() => onDelete(order.id)}
              className="flex items-center space-x-1 text-red-600 hover:text-red-700 hover:border-red-300"
            >
              <Trash2 className="h-4 w-4" />
              <span>Delete</span>
            </Button>
          )}
        </div>
      </CardContent>
    </Card>
  );
};

export default OrderCard;
